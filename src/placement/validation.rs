use bevy::prelude::*;

use crate::objects::components::*;
use crate::placement::footprints_intersect;
use crate::store::{PlacementInvalidReason, StoreArea, WorldBounds, validate_polygon_in_store};
use crate::tools::{ActiveToolAction, ToolContext};

pub fn validate_active_placement(
    mut tool: ResMut<ToolContext>,
    world: Res<WorldBounds>,
    store: Res<StoreArea>,
    footprints: Query<(Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>)>,
) {
    let Some(active) = tool.active.as_mut() else {
        return;
    };

    let (moving_entity, candidate_pos, active_footprint) = match active {
        ActiveToolAction::Moving {
            entity,
            current_world_pos,
            ..
        } => {
            let Ok((_, _, footprint, _)) = footprints.get(*entity) else {
                return;
            };
            (Some(*entity), *current_world_pos, footprint.clone())
        }
        ActiveToolAction::Building {
            ghost,
            current_world_pos,
            ..
        } => {
            let Ok((_, _, footprint, _)) = footprints.get(*ghost) else {
                return;
            };
            (Some(*ghost), *current_world_pos, footprint.clone())
        }
        ActiveToolAction::PendingDelete { .. } => return,
    };

    let store_valid =
        validate_polygon_in_store(&world, &store, &active_footprint, candidate_pos).is_ok();

    let intersects_blocker = footprints
        .iter()
        .filter(|(entity, _, _, blocks)| Some(*entity) != moving_entity && blocks.is_some())
        .any(|(_, other_pos, other_footprint, _)| {
            footprints_intersect(
                &active_footprint,
                candidate_pos,
                other_footprint,
                other_pos.0,
            )
        });

    let valid = store_valid && !intersects_blocker;
    if !valid {
        let reason = if !store_valid {
            PlacementInvalidReason::OutsideOwnedStoreArea
        } else {
            PlacementInvalidReason::IntersectsBlockingObject
        };
        let _ = reason;
    }

    match active {
        ActiveToolAction::Moving { valid: v, .. } => *v = valid,
        ActiveToolAction::Building { valid: v, .. } => *v = valid,
        ActiveToolAction::PendingDelete { .. } => {}
    }
}
