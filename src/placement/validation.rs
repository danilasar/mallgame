use bevy::prelude::*;

use crate::objects::components::*;
use crate::placement::footprints_intersect;
use crate::store::{PlacementInvalidReason, StoreArea, WorldBounds, validate_polygon_in_store};

#[derive(Debug, Clone, Default)]
pub struct PlacementValidationOptions {
    pub ignore_entity: Option<Entity>,
}

pub fn validate_placement(
    world_bounds: &WorldBounds,
    store_area: &StoreArea,
    footprints: &Query<(Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>)>,
    active_footprint: &Footprint,
    candidate_pos: Vec2,
    options: PlacementValidationOptions,
) -> Result<(), PlacementInvalidReason> {
    // 1. Check world and store area bounds
    validate_polygon_in_store(world_bounds, store_area, active_footprint, candidate_pos)?;

    // 2. Check for intersections with blocking objects
    let intersects_blocker = footprints
        .iter()
        .filter(|(entity, _, _, blocks)| {
            blocks.is_some() && Some(*entity) != options.ignore_entity
        })
        .any(|(_, other_pos, other_footprint, _)| {
            footprints_intersect(
                active_footprint,
                candidate_pos,
                other_footprint,
                other_pos.0,
            )
        });

    if intersects_blocker {
        return Err(PlacementInvalidReason::IntersectsBlockingObject);
    }

    Ok(())
}
