use bevy::prelude::*;

use crate::objects::components::*;
use crate::placement::footprints_intersect;
use crate::tools::{ActiveToolAction, ToolContext};

pub fn validate_active_placement(
    mut tool: ResMut<ToolContext>,
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

    let valid = !footprints
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

    match active {
        ActiveToolAction::Moving { valid: v, .. } => *v = valid,
        ActiveToolAction::Building { valid: v, .. } => *v = valid,
        ActiveToolAction::PendingDelete { .. } => {}
    }
}
