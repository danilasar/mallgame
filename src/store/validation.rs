use bevy::prelude::*;

use crate::objects::components::Footprint;
use crate::placement::world_polygon;
use crate::store::{StoreArea, WorldBounds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementInvalidReason {
    IntersectsBlockingObject,
    OutsideOwnedStoreArea,
    OutsideWorldBounds,
}

pub fn validate_polygon_in_store(
    world: &WorldBounds,
    store: &StoreArea,
    footprint: &Footprint,
    candidate_pos: Vec2,
) -> Result<(), PlacementInvalidReason> {
    let polygon = world_polygon(footprint, candidate_pos);
    if polygon
        .iter()
        .any(|point| !rect_contains_point(world.rect, *point))
    {
        return Err(PlacementInvalidReason::OutsideWorldBounds);
    }
    if !store.contains_polygon(&polygon) {
        return Err(PlacementInvalidReason::OutsideOwnedStoreArea);
    }
    Ok(())
}

fn rect_contains_point(rect: Rect, point: Vec2) -> bool {
    point.x >= rect.min.x && point.x <= rect.max.x && point.y >= rect.min.y && point.y <= rect.max.y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::components::Footprint;
    use crate::store::StoreArea;

    #[test]
    fn placement_outside_owned_store_area_is_invalid() {
        let world = WorldBounds::default();
        let store = StoreArea::new(Vec2::ZERO);
        let footprint = Footprint::rectangle(Vec2::splat(8.0));

        assert_eq!(
            validate_polygon_in_store(&world, &store, &footprint, Vec2::new(32.0, -32.0)),
            Err(PlacementInvalidReason::OutsideOwnedStoreArea)
        );
        assert!(
            validate_polygon_in_store(&world, &store, &footprint, Vec2::new(-32.0, -32.0)).is_ok()
        );
    }
}
