use bevy::prelude::*;

use crate::input::PointerContext;
use crate::objects::components::*;
use crate::tools::NonInteractive;
use crate::presentation::{IsoProjection, world_to_iso};

pub fn update_hovered_object(
    projection: Res<IsoProjection>,
    mut pointer: ResMut<PointerContext>,
    query: Query<
        (
            Entity,
            &WorldPos,
            &FootAnchor,
            &Sprite,
            &Transform,
            &SortLayer,
        ),
        (With<Interactive>, Without<NonInteractive>),
    >,
) {
    if !pointer.has_pointer || pointer.over_ui {
        pointer.hovered_entity = None;
        return;
    }

    let mut hit: Option<(Entity, f32)> = None;
    for (entity, world_pos, foot_anchor, sprite, transform, layer) in &query {
        let Some(size) = sprite.custom_size else {
            continue;
        };

        let foot_projected = world_to_iso(world_pos.0, *projection);
        let sprite_center = foot_projected - foot_anchor.0;
        let local = pointer.projected_pos - sprite_center;
        let half = size * 0.5;

        if local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y {
            let rank = layer.base_z() + transform.translation.z;
            if hit.map_or(true, |(_, best_rank)| rank > best_rank) {
                hit = Some((entity, rank));
            }
        }
    }

    pointer.hovered_entity = hit.map(|(entity, _)| entity);
}
