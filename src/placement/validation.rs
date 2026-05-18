use bevy::prelude::*;

use crate::objects::components::*;
use crate::placement::world_polygon;
use crate::store::{PlacementInvalidReason, StoreArea, WorldBounds};

#[derive(Debug, Clone, Default)]
pub struct PlacementValidationOptions {
    pub ignore_entity: Option<Entity>,
    pub ignore_stable_id: Option<StableObjectId>,
}

pub fn validate_placement(
    world_bounds: &WorldBounds,
    store_area: &StoreArea,
    footprints: &Query<
        (Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>),
        Without<WallMounted>,
    >,
    access_zones: &Query<(Entity, &InteriorAccessZone, Option<&ObjectStableId>)>,
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

    // 3. Check for intersections with access zones
    let intersects_access = access_zones
        .iter()
        .filter(|(entity, _, stable_id)| {
            Some(*entity) != options.ignore_entity
                && stable_id.map(|id| id.0) != options.ignore_stable_id
        })
        .any(|(_, zone, _)| crate::placement::polygon_intersects_access_zone(&polygon, zone));

    if intersects_access {
        return Err(PlacementInvalidReason::DoorAccessBlocked);
    }

    Ok(())
}

pub fn validate_derived_door_placement(
    derived: &DerivedDoorPlacement,
    store_area: &StoreArea,
    footprints: &Query<
        (Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>),
        Without<WallMounted>,
    >,
    wallprints: &Query<(&Wallprint, &ObjectStableId)>,
    options: PlacementValidationOptions,
) -> Result<(), PlacementInvalidReason> {
    // 1. Check wall overlap
    for (existing_print, existing_id) in wallprints.iter() {
        if options.ignore_stable_id != Some(existing_id.0)
            && wallprints_conflict(&derived.wallprint, existing_print)
        {
            return Err(PlacementInvalidReason::WallMountedOverlap);
        }
    }

    // 2. Check access zone fits in StoreArea
    let coverage = store_area.contains_polygon_sampled(
        &derived.interior_access_zone.polygon,
        crate::store::CoverageSamplingOptions {
            max_edge_step: store_area.cell_size.x * 0.5,
            epsilon: 0.001,
        },
    );

    if !coverage.valid {
        return Err(PlacementInvalidReason::OutsideOwnedStoreArea);
    }

    // 3. Check access zone doesn't overlap blocking floor footprints
    let intersects_blocker = footprints
        .iter()
        .filter(|(entity, _, _, blocks)| blocks.is_some() && Some(*entity) != options.ignore_entity)
        .any(|(_, other_pos, other_footprint, _)| {
            let other_world_polygon = crate::placement::world_polygon(other_footprint, other_pos.0);
            crate::placement::polygon_intersects_access_zone(
                &other_world_polygon,
                &derived.interior_access_zone,
            )
        });

    if intersects_blocker {
        return Err(PlacementInvalidReason::DoorAccessBlocked);
    }

    Ok(())
}
