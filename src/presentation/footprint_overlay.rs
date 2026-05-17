use bevy::prelude::*;

use crate::objects::components::{Footprint, Movable, WorldPos};
use crate::placement::world_polygon;
use crate::presentation::IsoProjection;
use crate::presentation::world_to_iso;
use crate::tools::{ActiveToolAction, ToolContext, ToolMode};

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
    tool: Res<ToolContext>,
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
    if *mode.get() != ToolMode::Move {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let target = match tool.active {
        Some(ActiveToolAction::Moving { entity, .. }) => Some(entity),
        _ => tool.hovered,
    };

    let Some(target) = target else {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let Ok((world_pos, footprint, movable)) = objects.get(target) else {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    if movable.is_none() {
        for (_, _, _, _, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let points = world_polygon(footprint, world_pos.0);
    if points.len() < 2 {
        return;
    }

    let mut segment_count = 0usize;
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

        if let Some((_, mut overlay, mut sprite, mut transform, mut visibility)) =
            overlays.iter_mut().nth(segment_count)
        {
            overlay.target = target;
            *sprite = Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0));
            transform.translation = Vec3::new(mid.x, mid.y, 950.0);
            transform.rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));
            *visibility = Visibility::Visible;
        } else {
            commands.spawn((
                Sprite::from_color(Color::srgb(1.0, 0.86, 0.18), Vec2::new(length, 6.0)),
                Transform {
                    translation: Vec3::new(mid.x, mid.y, 950.0),
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

    for (_, _, _, _, mut visibility) in overlays.iter_mut().skip(segment_count) {
        *visibility = Visibility::Hidden;
    }
}
