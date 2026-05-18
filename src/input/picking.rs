use bevy::prelude::*;

use crate::input::PointerContext;
use crate::objects::components::*;
use crate::tools::NonInteractive;
use crate::presentation::{IsoProjection, world_to_iso};

#[derive(Resource, Default, Debug)]
pub struct PointerTargets {
    pub world_object: Option<Entity>,
    pub world_widget: Option<Entity>,
    pub debug: Option<Entity>,
}

pub fn update_hovered_object(
    projection: Res<IsoProjection>,
    mut pointer: ResMut<PointerContext>,
    mut targets: ResMut<PointerTargets>,
    query: Query<
        (
            Entity,
            &WorldPos,
            &FootAnchor,
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
    let mut hit_debug: Option<(Entity, f32)> = None;

    for (entity, world_pos, foot_anchor, sprite, transform, layer, role, _selectable) in &query {
        let Some(size) = sprite.custom_size else {
            continue;
        };

        let foot_projected = world_to_iso(world_pos.0, *projection);
        let sprite_center = foot_projected - foot_anchor.0;
        let local = pointer.projected_pos - sprite_center;
        let half = size * 0.5;

        if local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y {
            let rank = layer.base_z() + transform.translation.z;
            match role {
                InteractionRole::WorldObject => {
                    if hit_object.map_or(true, |(_, r)| rank > r) {
                        hit_object = Some((entity, rank));
                    }
                }
                InteractionRole::WorldWidget => {
                    if hit_widget.map_or(true, |(_, r)| rank > r) {
                        hit_widget = Some((entity, rank));
                    }
                }
                InteractionRole::Overlay | InteractionRole::Debug => {
                    if hit_debug.map_or(true, |(_, r)| rank > r) {
                        hit_debug = Some((entity, rank));
                    }
                }
                _ => {}
            }
        }
    }

    targets.world_object = hit_object.map(|(e, _)| e);
    targets.world_widget = hit_widget.map(|(e, _)| e);
    targets.debug = hit_debug.map(|(e, _)| e);

    // If we are over a widget, we DON'T want picking to fail for the world object behind it
    // because that's usually the object the widget belongs to.
    if let Some(widget) = targets.world_widget {
        pointer.hovered_entity = Some(widget);
    } else {
        pointer.hovered_entity = targets.world_object;
    }
}
