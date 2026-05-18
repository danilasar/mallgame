use super::*;
use crate::store::chunks::StoreChunkKind;

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
        Some(StoreChunkPurchaseRejectReason::AlreadyOwned)
    );
    assert_eq!(
        validate_chunk_purchase(
            &world,
            &store,
            StoreChunkCoord { x: -7, y: -7 },
            StoreChunkKind::Default
        )
        .reason,
        Some(StoreChunkPurchaseRejectReason::NotSideAdjacent)
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
        Some(StoreChunkPurchaseRejectReason::DirectionNotAllowed)
    );
    assert_eq!(
        validate_chunk_purchase(
            &world,
            &store,
            StoreChunkCoord { x: -1, y: 0 },
            StoreChunkKind::Default
        )
        .reason,
        Some(StoreChunkPurchaseRejectReason::DirectionNotAllowed)
    );
}

#[test]
fn side_adjacency_check() {
    let store = StoreArea::new(Vec2::ZERO);
    // Initial store has x: -5..0, y: -4..0
    assert!(is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: 0, y: -1 }
    )); // Right
    assert!(is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: -6, y: -1 }
    )); // Left
    assert!(is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: -1, y: 0 }
    )); // Up
    assert!(is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: -1, y: -5 }
    )); // Down
    assert!(!is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: 1, y: 1 }
    )); // Far away
    assert!(!is_side_adjacent_to_owned(
        &store,
        StoreChunkCoord { x: 0, y: 0 }
    )); // Diagonal
}

#[test]
fn hole_prevention_in_purchase_validation() {
    let world = WorldBounds::default();
    let mut store = StoreArea::new(Vec2::ZERO);

    // We need to enable more directions for this test
    store.expansion_policy.allow_right = true;
    store.expansion_policy.allow_up = true;

    // Initial chunks: (-5..-1, -4..-1)
    store.owned_chunks.insert(
        StoreChunkCoord { x: 0, y: -1 },
        crate::store::chunks::StoreChunkData {
            kind: StoreChunkKind::Default,
        },
    );
    store.owned_chunks.insert(
        StoreChunkCoord { x: 0, y: 0 },
        crate::store::chunks::StoreChunkData {
            kind: StoreChunkKind::Default,
        },
    );
    store.owned_chunks.insert(
        StoreChunkCoord { x: -1, y: 0 },
        crate::store::chunks::StoreChunkData {
            kind: StoreChunkKind::Default,
        },
    );

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
        store.owned_chunks.insert(
            coord,
            crate::store::chunks::StoreChunkData {
                kind: StoreChunkKind::Default,
            },
        );
    }

    // Policy must allow the candidate
    store.expansion_policy.allow_right = true;
    store.expansion_policy.allow_up = true;
    store.expansion_policy.allow_left = true;
    store.expansion_policy.allow_down = true;

    // With the more flexible boundary-based direction policy,
    // hole-creating configurations now correctly reach the hole check.
    assert_eq!(
        validate_chunk_purchase(
            &world,
            &store,
            StoreChunkCoord { x: 1, y: 1 },
            StoreChunkKind::Default
        )
        .reason,
        Some(StoreChunkPurchaseRejectReason::WouldCreateHole)
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
    store.owned_chunks.insert(
        coord1,
        crate::store::chunks::StoreChunkData {
            kind: StoreChunkKind::Default,
        },
    );

    // 2. Buy another chunk in the SAME new column (x = -6)
    // This used to fail with DirectionNotAllowed because the frontier moved to -6
    let coord2 = StoreChunkCoord { x: -6, y: -2 };
    assert!(validate_chunk_purchase(&world, &store, coord2, StoreChunkKind::Default).valid);
}
