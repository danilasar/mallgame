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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPurchaseValidation {
    pub coord: StoreChunkCoord,
    pub valid: bool,
    pub reason: Option<StoreChunkPurchaseInvalidReason>,
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
        let validation = validate_chunk_purchase(&world, &store, event.coord, event.kind);
        if validation.valid {
            store
                .owned_chunks
                .insert(event.coord, StoreChunkData { kind: event.kind });
            info!(
                "Purchased store chunk {:?}; owned_count={}",
                event.coord,
                store.owned_chunks.len()
            );
        } else {
            info!(
                "Rejected store chunk purchase {:?}: {:?}",
                event.coord, validation.reason
            );
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

    let expands_left = coord.x < bounds.min.x;
    let expands_right = coord.x > bounds.max.x;
    let expands_down = coord.y < bounds.min.y;
    let expands_up = coord.y > bounds.max.y;

    if expands_left && !policy.allow_left {
        return false;
    }
    if expands_right && !policy.allow_right {
        return false;
    }
    if expands_down && !policy.allow_down {
        return false;
    }
    if expands_up && !policy.allow_up {
        return false;
    }

    true
}

pub fn validate_chunk_purchase(
    world: &WorldBounds,
    store: &StoreArea,
    coord: StoreChunkCoord,
    _kind: StoreChunkKind,
) -> ChunkPurchaseValidation {
    let mut validation = ChunkPurchaseValidation {
        coord,
        valid: true,
        reason: None,
    };

    if store.owned_chunks.contains_key(&coord) {
        validation.valid = false;
        validation.reason = Some(StoreChunkPurchaseInvalidReason::AlreadyOwned);
        return validation;
    }
    if !rect_contains_rect(world.rect, store.chunk_rect(coord)) {
        validation.valid = false;
        validation.reason = Some(StoreChunkPurchaseInvalidReason::OutsideWorldBounds);
        return validation;
    }
    if store.expansion_policy.require_side_adjacency && !is_side_adjacent_to_owned(store, coord) {
        validation.valid = false;
        validation.reason = Some(StoreChunkPurchaseInvalidReason::NotSideAdjacent);
        return validation;
    }
    if !is_direction_allowed(store, coord) {
        validation.valid = false;
        validation.reason = Some(StoreChunkPurchaseInvalidReason::DirectionNotAllowed);
        return validation;
    }
    if store.expansion_policy.forbid_holes
        && crate::store::would_create_hole(&store.owned_chunks, coord)
    {
        validation.valid = false;
        validation.reason = Some(StoreChunkPurchaseInvalidReason::WouldCreateHole);
        return validation;
    }
    validation
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
            )
            .reason,
            Some(StoreChunkPurchaseInvalidReason::AlreadyOwned)
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -7, y: -7 },
                StoreChunkKind::Default
            )
            .reason,
            Some(StoreChunkPurchaseInvalidReason::NotSideAdjacent)
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
            .valid
        );
        assert!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -1, y: -5 },
                StoreChunkKind::Default
            )
            .valid
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: 0, y: -1 },
                StoreChunkKind::Default
            )
            .reason,
            Some(StoreChunkPurchaseInvalidReason::DirectionNotAllowed)
        );
        assert_eq!(
            validate_chunk_purchase(
                &world,
                &store,
                StoreChunkCoord { x: -1, y: 0 },
                StoreChunkKind::Default
            )
            .reason,
            Some(StoreChunkPurchaseInvalidReason::DirectionNotAllowed)
        );
    }

    #[test]
    fn side_adjacency_check() {
        let store = StoreArea::new(Vec2::ZERO);
        // Initial store has x: -5..0, y: -4..0
        assert!(is_side_adjacent_to_owned(&store, StoreChunkCoord { x: 0, y: -1 })); // Right
        assert!(is_side_adjacent_to_owned(&store, StoreChunkCoord { x: -6, y: -1 })); // Left
        assert!(is_side_adjacent_to_owned(&store, StoreChunkCoord { x: -1, y: 0 })); // Up
        assert!(is_side_adjacent_to_owned(&store, StoreChunkCoord { x: -1, y: -5 })); // Down
        assert!(!is_side_adjacent_to_owned(&store, StoreChunkCoord { x: 1, y: 1 })); // Far away
        assert!(!is_side_adjacent_to_owned(&store, StoreChunkCoord { x: 0, y: 0 })); // Diagonal
    }

    #[test]
    fn hole_prevention_in_purchase_validation() {
        let world = WorldBounds::default();
        let mut store = StoreArea::new(Vec2::ZERO);
        
        // We need to enable more directions for this test
        store.expansion_policy.allow_right = true;
        store.expansion_policy.allow_up = true;
        
        // Initial chunks: (-5..-1, -4..-1)
        store.owned_chunks.insert(StoreChunkCoord { x: 0, y: -1 }, StoreChunkData { kind: StoreChunkKind::Default });
        store.owned_chunks.insert(StoreChunkCoord { x: 0, y: 0 }, StoreChunkData { kind: StoreChunkKind::Default });
        store.owned_chunks.insert(StoreChunkCoord { x: -1, y: 0 }, StoreChunkData { kind: StoreChunkKind::Default });
        
        // Let's use the detects_simple_enclosed_hole logic from chunks.rs but via validate_chunk_purchase
        store.owned_chunks.clear();
        for coord in [
            StoreChunkCoord { x: -1, y: -1 },
            StoreChunkCoord { x: 0, y: -1 },
            StoreChunkCoord { x: 1, y: -1 },
            StoreChunkCoord { x: -1, y: 0 },
            StoreChunkCoord { x: 1, y: 0 },
            StoreChunkCoord { x: -1, y: 1 },
            StoreChunkCoord { x: 0, y: 1 },
        ] {
            store.owned_chunks.insert(coord, StoreChunkData { kind: StoreChunkKind::Default });
        }
        
        // Policy must allow the candidate
        store.expansion_policy.allow_right = true;
        store.expansion_policy.allow_up = true;
        store.expansion_policy.allow_left = true;
        store.expansion_policy.allow_down = true;

        // With the more flexible boundary-based direction policy, 
        // hole-creating configurations now correctly reach the hole check.
        assert_eq!(
            validate_chunk_purchase(&world, &store, StoreChunkCoord { x: 1, y: 1 }, StoreChunkKind::Default).reason,
            Some(StoreChunkPurchaseInvalidReason::WouldCreateHole)
        );
    }

    #[test]
    fn allows_filling_rows_after_expanding_frontier() {
        let world = WorldBounds::default();
        let mut store = StoreArea::new(Vec2::ZERO);
        // Initial bounds: (-5, -4) to (-1, -1)
        
        // 1. Expand Left by one chunk
        let coord1 = StoreChunkCoord { x: -6, y: -1 };
        assert!(validate_chunk_purchase(&world, &store, coord1, StoreChunkKind::Default).valid);
        
        // Apply it
        store.owned_chunks.insert(coord1, StoreChunkData { kind: StoreChunkKind::Default });
        
        // 2. Buy another chunk in the SAME new column (x = -6)
        // This used to fail with DirectionNotAllowed because the frontier moved to -6
        let coord2 = StoreChunkCoord { x: -6, y: -2 };
        assert!(validate_chunk_purchase(&world, &store, coord2, StoreChunkKind::Default).valid);
    }
}
