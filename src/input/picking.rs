use bevy::prelude::*;

use crate::input::PointerContext;
use crate::objects::components::*;
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{WallSegmentKey, WallSurface};
use crate::tools::NonInteractive;

#[derive(Resource, Default, Debug)]
pub struct PointerTargets {
    pub world_object: Option<Entity>,
    pub world_widget: Option<Entity>,
    pub wall_surface: Option<Entity>,
    pub exterior: Option<Entity>,
    pub debug: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallSurfaceHit {
    pub entity: Entity,
    pub key: WallSegmentKey,
    pub world_pos: Vec2,
    pub offset_along_segment: f32,
    pub height_on_wall: f32,
    pub normal: Vec2,
}

#[allow(clippy::type_complexity)]
pub fn update_hovered_object(
    projection: Res<IsoProjection>,
    mut pointer: ResMut<PointerContext>,
    mut targets: ResMut<PointerTargets>,
    query: Query<
        (
            Entity,
            Option<&WorldPos>,
            Option<&FootAnchor>,
            Option<&VisualOffset>,
            Option<&WallSurface>,
            Option<&Sprite>,
            &Transform,
            &SortLayer,
            &InteractionRole,
            Option<&Selectable>,
        ),
        (With<Interactive>, Without<NonInteractive>),
    >,
) {
    targets.world_object = None;
    targets.world_widget = None;
    targets.wall_surface = None;
    targets.exterior = None;
    targets.debug = None;

    // Invariant: PointerContext.over_ui is set by the UI system if cursor is over standard UI elements.
    // However, if we are over a WorldWidget, we might still want to maintain the hovered world object
    // to prevent flickering.
    if !pointer.has_pointer || (pointer.over_ui && targets.world_widget.is_none()) {
        // If we were ALREADY over a widget last frame, we might skip the early return.
        // But picking happens before UI interaction is updated.
        // Actually, the UI system update_pointer_over_ui should be more selective.
    }

    if !pointer.has_pointer {
        pointer.hovered_entity = None;
        return;
    }

    let mut hit_object: Option<(Entity, f32)> = None;
    let mut hit_widget: Option<(Entity, f32)> = None;
    let mut hit_wall_surface: Option<(Entity, f32)> = None;
    let mut hit_exterior: Option<(Entity, f32)> = None;
    let mut hit_debug: Option<(Entity, f32)> = None;

    for (
        entity,
        world_pos,
        foot_anchor,
        visual_offset,
        wall_surface,
        sprite,
        transform,
        layer,
        role,
        _selectable,
    ) in &query
    {
        let hit = if let Some(surface) = wall_surface {
            wall_surface_hit(pointer.projected_pos, *projection, entity, surface).is_some()
        } else {
            object_hit(
                pointer.projected_pos,
                *projection,
                world_pos,
                foot_anchor,
                visual_offset,
                sprite,
            )
        };

        if hit {
            let rank = layer.base_z() + transform.translation.z;
            match role {
                InteractionRole::WorldObject if hit_object.is_none_or(|(_, r)| rank > r) => {
                    hit_object = Some((entity, rank));
                }
                InteractionRole::WorldWidget if hit_widget.is_none_or(|(_, r)| rank > r) => {
                    hit_widget = Some((entity, rank));
                }
                InteractionRole::WallSurface if hit_wall_surface.is_none_or(|(_, r)| rank > r) => {
                    hit_wall_surface = Some((entity, rank));
                }
                InteractionRole::Exterior if hit_exterior.is_none_or(|(_, r)| rank > r) => {
                    hit_exterior = Some((entity, rank));
                }
                InteractionRole::Overlay | InteractionRole::Debug
                    if hit_debug.is_none_or(|(_, r)| rank > r) =>
                {
                    hit_debug = Some((entity, rank));
                }
                _ => {}
            }
        }
    }

    targets.world_object = hit_object.map(|(e, _)| e);
    targets.world_widget = hit_widget.map(|(e, _)| e);
    targets.wall_surface = hit_wall_surface.map(|(e, _)| e);
    targets.exterior = hit_exterior.map(|(e, _)| e);
    targets.debug = hit_debug.map(|(e, _)| e);

    // If we are over a widget, we DON'T want picking to fail for the world object behind it
    // because that's usually the object the widget belongs to.
    if let Some(widget) = targets.world_widget {
        pointer.hovered_entity = Some(widget);
    } else {
        pointer.hovered_entity = targets.world_object;
    }
}

fn object_hit(
    projected_pos: Vec2,
    projection: IsoProjection,
    world_pos: Option<&WorldPos>,
    foot_anchor: Option<&FootAnchor>,
    visual_offset: Option<&VisualOffset>,
    sprite: Option<&Sprite>,
) -> bool {
    let (Some(world_pos), Some(foot_anchor)) = (world_pos, foot_anchor) else {
        return false;
    };
    let Some(sprite) = sprite else {
        return false;
    };
    let Some(size) = sprite.custom_size else {
        return false;
    };

    let foot_projected = world_to_iso(world_pos.0, projection);
    let sprite_center = foot_projected - foot_anchor.0 + visual_offset.map_or(Vec2::ZERO, |v| v.0);
    let local = projected_pos - sprite_center;
    let half = size * 0.5;

    local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y
}

pub fn wall_surface_hit(
    projected_pos: Vec2,
    projection: IsoProjection,
    entity: Entity,
    wall_surface: &WallSurface,
) -> Option<WallSurfaceHit> {
    let projected_start = world_to_iso(wall_surface.start, projection);
    let projected_end = world_to_iso(wall_surface.end, projection);
    let segment = projected_end - projected_start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return None;
    }

    let wall_direction = segment.normalize();
    let wall_normal = Vec2::new(-wall_direction.y, wall_direction.x);
    let thickness_offset = wall_normal * wall_surface.thickness;
    let quad = [
        projected_start,
        projected_end,
        projected_end + thickness_offset + Vec2::new(0.0, wall_surface.height),
        projected_start + thickness_offset + Vec2::new(0.0, wall_surface.height),
    ];

    if !point_in_convex_quad(projected_pos, quad) {
        return None;
    }

    let relative = projected_pos - projected_start;
    let along = relative.dot(wall_direction);
    let projected_length = segment.length();
    let t = (along / projected_length).clamp(0.0, 1.0);
    let offset_along_segment = (wall_surface.length * t).clamp(0.0, wall_surface.length);
    let base_world = wall_surface.start.lerp(wall_surface.end, t);
    let base_projected = projected_start.lerp(projected_end, t);
    let surface_base_projected = base_projected + thickness_offset;
    let height_on_wall =
        (projected_pos.y - surface_base_projected.y).clamp(0.0, wall_surface.height);

    Some(WallSurfaceHit {
        entity,
        key: wall_surface.key,
        world_pos: base_world,
        offset_along_segment,
        height_on_wall,
        normal: wall_surface.normal,
    })
}

pub(crate) fn point_in_convex_quad(point: Vec2, quad: [Vec2; 4]) -> bool {
    let mut last_sign = 0.0;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let edge = b - a;
        let to_point = point - a;
        let cross = edge.perp_dot(to_point);
        if cross.abs() <= f32::EPSILON {
            continue;
        }
        let sign = cross.signum();
        if last_sign == 0.0 {
            last_sign = sign;
        } else if sign != last_sign {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::IsoProjection;
    use crate::store::{StoreChunkCoord, WallSegmentKey, WallSurface};

    #[test]
    fn wall_surface_hit_projects_and_clamps_to_surface() {
        let projection = IsoProjection::default();
        let surface = WallSurface {
            key: WallSegmentKey {
                chunk: StoreChunkCoord { x: 0, y: 0 },
                side: crate::store::boundary::StoreBoundarySide::Top,
            },
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(4.0, 0.0),
            length: 4.0,
            height: 6.0,
            thickness: 2.0,
            normal: Vec2::Y,
        };

        let projected_start = world_to_iso(surface.start, projection);
        let projected_end = world_to_iso(surface.end, projection);
        let projected_mid = projected_start.lerp(projected_end, 0.5);
        let wall_direction = (projected_end - projected_start).normalize();
        let wall_normal = Vec2::new(-wall_direction.y, wall_direction.x);
        let projected_mid_on_face = projected_mid + wall_normal * surface.thickness;

        let hit = wall_surface_hit(
            projected_mid_on_face,
            projection,
            Entity::from_bits(1),
            &surface,
        )
        .expect("expected a wall hit");

        assert_eq!(hit.key, surface.key);
        assert!((hit.offset_along_segment - 2.0).abs() < 0.001);
        assert!(hit.height_on_wall >= 0.0 && hit.height_on_wall <= surface.height);
        assert_eq!(hit.normal, Vec2::Y);
        assert!((hit.world_pos - Vec2::new(2.0, 0.0)).length() < 0.001);

        let elevated = projected_mid_on_face + Vec2::new(0.0, 0.5);
        let elevated_hit = wall_surface_hit(elevated, projection, Entity::from_bits(2), &surface)
            .expect("expected elevated wall hit");
        assert!(elevated_hit.height_on_wall > 0.0);

        let wall_face = projected_mid_on_face + wall_normal * 0.5;
        assert!(wall_surface_hit(wall_face, projection, Entity::from_bits(4), &surface).is_some());

        let behind_wall = projected_mid_on_face - wall_normal * 2.5;
        assert!(
            wall_surface_hit(behind_wall, projection, Entity::from_bits(5), &surface).is_none()
        );

        let outside = projected_mid_on_face + wall_direction * 5.0;
        assert!(wall_surface_hit(outside, projection, Entity::from_bits(3), &surface).is_none());
    }
}
