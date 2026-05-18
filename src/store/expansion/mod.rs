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
mod tests;
