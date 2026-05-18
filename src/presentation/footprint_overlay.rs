use bevy::prelude::*;

use crate::objects::components::{
    Footprint, InteractionRole, RuntimeOwned, RuntimeOwner, SortLayer, WallMounted,
    WorldPos, Wallprint,
};
use crate::placement::world_polygon;
use crate::presentation::IsoProjection;
use crate::presentation::world_to_iso;
use crate::store::{WallSurface, wall_surface_visual_offset, wall_surface_world_pos};
use crate::tools::{NonInteractive, ToolMode};

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

pub fn update_footprint_outline_overlay(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    session: Res<crate::tools::ToolSessionState>,
    tool: Res<crate::tools::ToolContext>,
    projection: Res<IsoProjection>,
    objects: Query<(&WorldPos, &Footprint), Without<WallMounted>>,
    wall_objects: Query<&Wallprint>,
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

    let mut points = Vec::new();

    if let Ok((world_pos, footprint)) = objects.get(target) {
        let world_points = world_polygon(footprint, world_pos.0);
        for p in world_points {
            points.push(world_to_iso(p, *projection));
        }
    } else if let Ok(wallprint) = wall_objects.get(target) {
        if let Some(rect) = wallprint.rects.first() {
            if let Some(surface) = wall_surfaces.iter().find(|s| s.key == rect.segment_key) {
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
        }
    }

    if points.len() < 2 {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let mut segment_count = 0usize;
    let existing_segments: Vec<_> = overlays.iter().map(|(e, _, _, _, _)| e).collect();

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
                *sprite = Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0));
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

    // Hide remaining unused segments
    for entity in existing_segments.iter().skip(segment_count).copied() {
        if let Ok((_, _, _, _, mut visibility)) = overlays.get_mut(entity) {
            *visibility = Visibility::Hidden;
        }
    }
}
