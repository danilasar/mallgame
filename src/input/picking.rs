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

    if !pointer.has_pointer || pointer.over_ui {
        pointer.hovered_entity = None;
        return;
    }

    let mut hit_object: Option<(Entity, f32)> = None;
    let mut hit_widget: Option<(Entity, f32)> = None;
    let mut hit_debug: Option<(Entity, f32)> = None;

    for (entity, world_pos, foot_anchor, sprite, transform, layer, role, selectable) in &query {
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
                    if selectable.is_some() && hit_object.map_or(true, |(_, best_rank)| rank > best_rank) {
                        hit_object = Some((entity, rank));
                    }
                }
                InteractionRole::WorldWidget => {
                    if hit_widget.map_or(true, |(_, best_rank)| rank > best_rank) {
                        hit_widget = Some((entity, rank));
                    }
                }
                InteractionRole::Debug => {
                    if hit_debug.map_or(true, |(_, best_rank)| rank > best_rank) {
                        hit_debug = Some((entity, rank));
                    }
                }
                InteractionRole::ToolPreview | InteractionRole::Overlay => {}
            }
        }
    }

    targets.world_object = hit_object.map(|(e, _)| e);
    targets.world_widget = hit_widget.map(|(e, _)| e);
    targets.debug = hit_debug.map(|(e, _)| e);

    // Legacy fallback or primary target: Widget > Object > Debug
    pointer.hovered_entity = targets.world_widget
        .or(targets.world_object)
        .or(targets.debug);
}
