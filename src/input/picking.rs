use bevy::prelude::*;

use crate::input::PointerContext;
use crate::objects::components::*;
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::WallSurface;
use crate::tools::NonInteractive;

#[derive(Resource, Default, Debug)]
pub struct PointerTargets {
    pub world_object: Option<Entity>,
    pub world_widget: Option<Entity>,
    pub wall_surface: Option<Entity>,
    pub exterior: Option<Entity>,
    pub debug: Option<Entity>,
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
            Option<&WallSurface>,
            &Sprite,
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
        wall_surface,
        sprite,
        transform,
        layer,
        role,
        _selectable,
    ) in &query
    {
        let hit = if let Some(surface) = wall_surface {
            wall_surface_hit(pointer.projected_pos, *projection, surface, sprite)
        } else {
            object_hit(pointer.projected_pos, *projection, world_pos, foot_anchor, sprite)
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
    sprite: &Sprite,
) -> bool {
    let (Some(world_pos), Some(foot_anchor)) = (world_pos, foot_anchor) else {
        return false;
    };
    let Some(size) = sprite.custom_size else {
        return false;
    };

    let foot_projected = world_to_iso(world_pos.0, projection);
    let sprite_center = foot_projected - foot_anchor.0;
    let local = projected_pos - sprite_center;
    let half = size * 0.5;

    local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y
}

fn wall_surface_hit(
    projected_pos: Vec2,
    projection: IsoProjection,
    wall_surface: &WallSurface,
    sprite: &Sprite,
) -> bool {
    let Some(size) = sprite.custom_size else {
        return false;
    };
    let projected_start = world_to_iso(wall_surface.start, projection);
    let projected_end = world_to_iso(wall_surface.end, projection);
    let distance = point_to_segment_distance(projected_pos, projected_start, projected_end);
    let thickness = size.y.max(wall_surface.thickness) * 0.5;
    distance <= thickness + 1.0
}

fn point_to_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    let projection = start + segment * t;
    point.distance(projection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_to_segment_distance_handles_degenerate_and_regular_segments() {
        assert!((point_to_segment_distance(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO) - 0.0).abs() < 0.0001);
        assert!((point_to_segment_distance(Vec2::new(2.0, 1.0), Vec2::ZERO, Vec2::new(4.0, 0.0)) - 1.0).abs() < 0.0001);
    }
}
