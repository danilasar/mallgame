use bevy::prelude::*;

use crate::store::{
    StoreArea, StoreChunkCoord, StoreChunkData, StoreChunkKind, WorldBounds, side_neighbors,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StoreChunkPurchaseInvalidReason {
    AlreadyOwned,
    OutsideWorldBounds,
    NotSideAdjacent,
    DirectionNotAllowed,
    WouldCreateHole,
    Locked,
    CannotAfford,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PurchaseStoreChunkRequested {
    pub coord: StoreChunkCoord,
    pub kind: StoreChunkKind,
}

pub fn apply_purchase_store_chunk_requested(
    mut events: MessageReader<PurchaseStoreChunkRequested>,
    mut store: ResMut<StoreArea>,
    world: Res<WorldBounds>,
) {
    for event in events.read() {
        match validate_chunk_purchase(&world, &store, event.coord, event.kind) {
            Ok(()) => {
                store
                    .owned_chunks
                    .insert(event.coord, StoreChunkData { kind: event.kind });
                info!(
                    "Purchased store chunk {:?}; owned_count={}",
                    event.coord,
                    store.owned_chunks.len()
                );
            }
            Err(reason) => {
                info!(
                    "Rejected store chunk purchase {:?}: {:?}",
                    event.coord, reason
                );
            }
        }
    }
}

pub fn is_side_adjacent_to_owned(store: &StoreArea, coord: StoreChunkCoord) -> bool {
    side_neighbors(coord)
        .iter()
        .any(|neighbor| store.owned_chunks.contains_key(neighbor))
}

pub fn is_direction_allowed(store: &StoreArea, coord: StoreChunkCoord) -> bool {
    let Some(bounds) = store.owned_chunk_bounds() else {
        return false;
    };
    let policy = store.expansion_policy;

    (policy.allow_left
        && coord.x == bounds.min.x - 1
        && coord.y >= bounds.min.y
        && coord.y <= bounds.max.y)
        || (policy.allow_right
            && coord.x == bounds.max.x + 1
            && coord.y >= bounds.min.y
            && coord.y <= bounds.max.y)
        || (policy.allow_down
            && coord.y == bounds.min.y - 1
            && coord.x >= bounds.min.x
            && coord.x <= bounds.max.x)
        || (policy.allow_up
            && coord.y == bounds.max.y + 1
            && coord.x >= bounds.min.x
            && coord.x <= bounds.max.x)
}

pub fn validate_chunk_purchase(
    world: &WorldBounds,
    store: &StoreArea,
    coord: StoreChunkCoord,
    _kind: StoreChunkKind,
) -> Result<(), StoreChunkPurchaseInvalidReason> {
    if store.owned_chunks.contains_key(&coord) {
        return Err(StoreChunkPurchaseInvalidReason::AlreadyOwned);
    }
    if !rect_contains_rect(world.rect, store.chunk_rect(coord)) {
        return Err(StoreChunkPurchaseInvalidReason::OutsideWorldBounds);
    }
    if store.expansion_policy.require_side_adjacency && !is_side_adjacent_to_owned(store, coord) {
        return Err(StoreChunkPurchaseInvalidReason::NotSideAdjacent);
    }
    if !is_direction_allowed(store, coord) {
        return Err(StoreChunkPurchaseInvalidReason::DirectionNotAllowed);
    }
    if store.expansion_policy.forbid_holes
        && crate::store::would_create_hole(&store.owned_chunks, coord)
    {
        return Err(StoreChunkPurchaseInvalidReason::WouldCreateHole);
    }
    Ok(())
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.min.x >= outer.min.x
        && inner.max.x <= outer.max.x
        && inner.min.y >= outer.min.y
        && inner.max.y <= outer.max.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_and_non_adjacent_are_invalid() {
        let world = WorldBounds::default();
        let store = StoreArea::new(Vec2::ZERO);
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -1, y: -1 },
                StoreChunkKind::Default
            ),
            Err(StoreChunkPurchaseInvalidReason::AlreadyOwned)
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -7, y: -7 },
                StoreChunkKind::Default
            ),
            Err(StoreChunkPurchaseInvalidReason::NotSideAdjacent)
        );
    }

    #[test]
    fn current_policy_allows_left_and_down_not_right_or_up() {
        let world = WorldBounds::default();
        let store = StoreArea::new(Vec2::ZERO);
        assert!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -6, y: -1 },
                StoreChunkKind::Default
            )
            .is_ok()
        );
        assert!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -1, y: -5 },
                StoreChunkKind::Default
            )
            .is_ok()
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: 0, y: -1 },
                StoreChunkKind::Default
            ),
            Err(StoreChunkPurchaseInvalidReason::DirectionNotAllowed)
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -1, y: 0 },
                StoreChunkKind::Default
            ),
            Err(StoreChunkPurchaseInvalidReason::DirectionNotAllowed)
        );
    }
}
