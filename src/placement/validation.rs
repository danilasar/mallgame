use bevy::prelude::*;

use crate::objects::components::*;
use crate::placement::world_polygon;
use crate::store::{PlacementInvalidReason, StoreArea, WorldBounds};

#[derive(Debug, Clone, Default)]
pub struct PlacementValidationOptions {
    pub ignore_entity: Option<Entity>,
}

pub fn validate_placement(
    world_bounds: &WorldBounds,
    store_area: &StoreArea,
    footprints: &Query<
        (Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>),
        Without<WallMounted>,
    >,
    active_footprint: &Footprint,
    candidate_pos: Vec2,
    options: PlacementValidationOptions,
) -> Result<(), PlacementInvalidReason> {
    // 1. Check world and store area bounds
    if !world_bounds.rect.contains(candidate_pos) {
        return Err(PlacementInvalidReason::OutsideWorldBounds);
    }

    let polygon = world_polygon(active_footprint, candidate_pos);
    let coverage = store_area.contains_polygon_sampled(
        &polygon,
        crate::store::CoverageSamplingOptions {
            max_edge_step: store_area.cell_size.x * 0.5,
            epsilon: 0.001,
        },
    );

    if !coverage.valid {
        if let Some(point) = coverage.failed_point {
            debug!("Placement failed at point {:?}", point);
        }
        return Err(PlacementInvalidReason::OutsideOwnedStoreArea);
    }

    // 2. Check for intersections with blocking objects
    let intersects_blocker = footprints
        .iter()
        .filter(|(entity, _, _, blocks)| blocks.is_some() && Some(*entity) != options.ignore_entity)
        .any(|(_, other_pos, other_footprint, _)| {
            crate::placement::footprints_intersect(
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
