use bevy::prelude::*;

use crate::objects::components::{
    AccessZonePreviewShape, Footprint, InteractionRole, RuntimeOwned, RuntimeOwner, SortLayer,
    WallMounted, Wallprint, WorldPos,
};
use crate::placement::world_polygon;
use crate::presentation::IsoProjection;
use crate::presentation::world_to_iso;
use crate::store::{WallSurface, wall_surface_visual_offset, wall_surface_world_pos};
use crate::tools::{ActiveToolSession, NonInteractive, ToolMode};

#[derive(Component, Debug, Clone, Copy)]
pub struct FootprintOutlineOverlay {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct FootprintOutlineSegment;

pub struct FootprintOverlayPlugin;

impl Plugin for FootprintOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_footprint_outline_overlay.before(TransformSystems::Propagate),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_footprint_outline_overlay(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    session: Res<crate::tools::ToolSessionState>,
    tool: Res<crate::tools::ToolContext>,
    projection: Res<IsoProjection>,
    objects: Query<(&WorldPos, &Footprint), Without<WallMounted>>,
    wall_objects: Query<&Wallprint>,
    access_zones: Query<&crate::objects::components::InteriorAccessZone>,
    access_zone_previews: Query<&AccessZonePreviewShape>,
    wall_surfaces: Query<&WallSurface>,
    mut overlays: Query<
        (
            Entity,
            &mut FootprintOutlineOverlay,
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
        ),
        With<FootprintOutlineSegment>,
    >,
) {
    // Abstracted target selection: Active Preview has priority over Hovered object
    let preview_target = session.active.as_ref().and_then(|s| s.preview_entity());

    // 2. If no active session, show outline on hover ONLY in Move mode (to indicate pickability)
    let hover_target = if preview_target.is_none() && *mode.get() == ToolMode::Move {
        tool.hovered_entity
    } else {
        None
    };

    let target = preview_target.or(hover_target);

    let Some(target) = target else {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let mut point_sets = Vec::new();

    if let Ok((world_pos, footprint)) = objects.get(target) {
        let mut points = Vec::new();
        let world_points = world_polygon(footprint, world_pos.0);
        for p in world_points {
            points.push(world_to_iso(p, *projection));
        }
        point_sets.push(points);
    }

    if let Ok(wallprint) = wall_objects.get(target) {
        let mut points = Vec::new();
        if let Some(rect) = wallprint.rects.first()
            && let Some(surface) = wall_surfaces.iter().find(|s| s.key == rect.segment_key)
        {
            let p1_world = wall_surface_world_pos(surface, rect.offset_min);
            let p2_world = wall_surface_world_pos(surface, rect.offset_max);

            let p1_iso = world_to_iso(p1_world, *projection);
            let p2_iso = world_to_iso(p2_world, *projection);

            let v_min = wall_surface_visual_offset(surface, *projection, rect.height_min);
            let v_max = wall_surface_visual_offset(surface, *projection, rect.height_max);

            points.push(p1_iso + v_min);
            points.push(p2_iso + v_min);
            points.push(p2_iso + v_max);
            points.push(p1_iso + v_max);
        }
        if !points.is_empty() {
            point_sets.push(points);
        }
    }

    if let Ok(access_zone) = access_zones.get(target) {
        let mut points = Vec::new();
        for &p in &access_zone.polygon {
            points.push(world_to_iso(p, *projection));
        }
        if !points.is_empty() {
            point_sets.push(points);
        }
    }

    // Special case: if we are building/moving a door, we might want to see the access zone
    // even if it's on a separate preview entity.
    if let Some(ActiveToolSession::Build(crate::tools::BuildToolSession::WallMounted(wall))) =
        session.active.as_ref()
    {
        if let Some(az_preview) = wall.access_zone_preview_entity
            && let Ok(access_zone) = access_zone_previews.get(az_preview)
        {
            let mut points = Vec::new();
            for &p in &access_zone.polygon {
                points.push(world_to_iso(p, *projection));
            }
            if !points.is_empty() {
                point_sets.push(points);
            }
        }
    } else if let Some(ActiveToolSession::Move(crate::tools::MoveToolSession::Door(door))) =
        session.active.as_ref()
        && let Some(derived) = &door.current_derived
    {
        let mut points = Vec::new();
        for &p in &derived.interior_access_zone.polygon {
            points.push(world_to_iso(p, *projection));
        }
        if !points.is_empty() {
            point_sets.push(points);
        }
    }

    if point_sets.is_empty() {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let mut segment_count = 0usize;
    let existing_segments: Vec<_> = overlays.iter().map(|(e, _, _, _, _)| e).collect();

    for points in point_sets {
        for (pa, pb) in points
            .iter()
            .copied()
            .zip(points.iter().copied().cycle().skip(1))
            .take(points.len())
        {
            let mid = (pa + pb) * 0.5;
            let delta = pb - pa;
            let length = delta.length();
            if length <= 0.1 {
                continue;
            }

            if segment_count < existing_segments.len() {
                let entity = existing_segments[segment_count];
                if let Ok((_, mut overlay, mut sprite, mut transform, mut visibility)) =
                    overlays.get_mut(entity)
                {
                    overlay.target = target;
                    *sprite =
                        Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0));
                    transform.translation =
                        Vec3::new(mid.x, mid.y, SortLayer::SelectionOverlay.base_z());
                    transform.rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));
                    *visibility = Visibility::Visible;
                }
            } else {
                commands.spawn((
                    Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0)),
                    Transform {
                        translation: Vec3::new(mid.x, mid.y, SortLayer::SelectionOverlay.base_z()),
                        rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                        ..default()
                    },
                    Visibility::Visible,
                    FootprintOutlineOverlay { target },
                    FootprintOutlineSegment,
                    InteractionRole::Overlay,
                    RuntimeOwned {
                        owner: RuntimeOwner::FootprintOverlay,
                    },
                    NonInteractive,
                    Name::new(format!("FootprintOutlineOverlay {:?}", target)),
                ));
            }
            segment_count += 1;
        }
    }

    // Hide remaining unused segments
    for entity in existing_segments.iter().skip(segment_count).copied() {
        if let Ok((_, _, _, _, mut visibility)) = overlays.get_mut(entity) {
            *visibility = Visibility::Hidden;
        }
    }
}
