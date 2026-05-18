use bevy::prelude::*;

use crate::store::{StoreArea, StoreChunkCoord, WorldBounds, owned_bounds, side_neighbors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StoreChunkPurchaseRejectReason {
    AlreadyOwned,
    NotSideAdjacent,
    OutsideWorldBounds,
    WouldCreateHole,
    DirectionNotAllowed,
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkPurchaseValidation {
    pub valid: bool,
    #[allow(dead_code)]
    pub reason: Option<StoreChunkPurchaseRejectReason>,
}

impl ChunkPurchaseValidation {
    pub fn ok() -> Self {
        Self {
            valid: true,
            reason: None,
        }
    }

    pub fn reject(reason: StoreChunkPurchaseRejectReason) -> Self {
        Self {
            valid: false,
            reason: Some(reason),
        }
    }
}

pub fn validate_chunk_purchase(
    world: &WorldBounds,
    store: &StoreArea,
    coord: StoreChunkCoord,
    _kind: crate::store::chunks::StoreChunkKind,
) -> ChunkPurchaseValidation {
    if store.owned_chunks.contains_key(&coord) {
        return ChunkPurchaseValidation::reject(StoreChunkPurchaseRejectReason::AlreadyOwned);
    }

    if !rect_contains_rect(world.rect, store.chunk_rect(coord)) {
        return ChunkPurchaseValidation::reject(StoreChunkPurchaseRejectReason::OutsideWorldBounds);
    }

    // Directional Policy Check
    if let Some(bounds) = owned_bounds(&store.owned_chunks) {
        let policy = store.expansion_policy;
        if !policy.allow_right && coord.x > bounds.max.x {
            return ChunkPurchaseValidation::reject(
                StoreChunkPurchaseRejectReason::DirectionNotAllowed,
            );
        }
        if !policy.allow_up && coord.y > bounds.max.y {
            return ChunkPurchaseValidation::reject(
                StoreChunkPurchaseRejectReason::DirectionNotAllowed,
            );
        }
        if !policy.allow_left && coord.x < bounds.min.x {
            return ChunkPurchaseValidation::reject(
                StoreChunkPurchaseRejectReason::DirectionNotAllowed,
            );
        }
        if !policy.allow_down && coord.y < bounds.min.y {
            return ChunkPurchaseValidation::reject(
                StoreChunkPurchaseRejectReason::DirectionNotAllowed,
            );
        }
    }

    if !is_side_adjacent_to_owned(store, coord) {
        return ChunkPurchaseValidation::reject(StoreChunkPurchaseRejectReason::NotSideAdjacent);
    }

    if would_create_hole(&store.owned_chunks, coord) {
        return ChunkPurchaseValidation::reject(StoreChunkPurchaseRejectReason::WouldCreateHole);
    }

    ChunkPurchaseValidation::ok()
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.min.x >= outer.min.x
        && inner.max.x <= outer.max.x
        && inner.min.y >= outer.min.y
        && inner.max.y <= outer.max.y
}

pub fn convert_purchase_requests_to_commands(
    mut events: MessageReader<crate::store::PurchaseStoreChunkRequested>,
    mut queue: ResMut<crate::store::commands::DomainCommandQueue>,
) {
    for event in events.read() {
        queue
            .commands
            .push_back(crate::store::commands::DomainCommand::PurchaseChunk(
                crate::store::commands::PurchaseChunkCommand {
                    coord: event.coord,
                    kind: event.kind,
                },
            ));
    }
}

pub fn is_side_adjacent_to_owned(store: &StoreArea, coord: StoreChunkCoord) -> bool {
    side_neighbors(coord)
        .iter()
        .any(|&neighbor| store.owned_chunks.contains_key(&neighbor))
}

pub fn would_create_hole(
    owned_chunks: &std::collections::HashMap<StoreChunkCoord, crate::store::chunks::StoreChunkData>,
    new_chunk: StoreChunkCoord,
) -> bool {
    crate::store::chunks::would_create_hole(owned_chunks, new_chunk)
}

#[cfg(test)]
mod tests {
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
}
