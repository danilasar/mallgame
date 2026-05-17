use bevy::prelude::*;

use crate::objects::components::{Footprint, Movable, SortLayer, WorldPos};
use crate::placement::world_polygon;
use crate::presentation::IsoProjection;
use crate::presentation::world_to_iso;
use crate::tools::ToolMode;

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
    objects: Query<(&WorldPos, &Footprint, Option<&Movable>)>,
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
    // 1. If there's an active session with a preview, it always gets the focus outline
    let preview_target = session.active.as_ref().and_then(|s| s.preview_entity());

    // 2. If no active session, show outline on hover ONLY in Move mode (to indicate pickability)
    let hover_target = if preview_target.is_none() && *mode.get() == ToolMode::Move {
        tool.hovered
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
    let Ok((world_pos, footprint, _movable)) = objects.get(target) else {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let points = world_polygon(footprint, world_pos.0);
    if points.len() < 2 {
        return;
    }

    let mut segment_count = 0usize;
    // We cannot easily collect Query results into a Vec for reuse due to mutable borrowing.
    // We'll use iter_mut() and nth() sparingly or just iterate once.
    
    // Convert to a reusable list of entities we already have
    let existing_segments: Vec<_> = overlays.iter().map(|(e, _, _, _, _)| e).collect();

    for (a, b) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let pa = world_to_iso(a, *projection);
        let pb = world_to_iso(b, *projection);
        let mid = (pa + pb) * 0.5;
        let delta = pb - pa;
        let length = delta.length();
        if length <= 0.1 {
            continue;
        }

        if segment_count < existing_segments.len() {
            let entity = existing_segments[segment_count];
            if let Ok((_, mut overlay, mut sprite, mut transform, mut visibility)) = overlays.get_mut(entity) {
                overlay.target = target;
                *sprite = Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0));
                transform.translation = Vec3::new(mid.x, mid.y, SortLayer::SelectionOverlay.base_z());
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
                Name::new(format!("FootprintOutlineOverlay {:?}", target)),
            ));
        }
        segment_count += 1;
    }

    // Hide remaining unused segments
    for i in segment_count..existing_segments.len() {
        if let Ok((_, _, _, _, mut visibility)) = overlays.get_mut(existing_segments[i]) {
            *visibility = Visibility::Hidden;
        }
    }
}
