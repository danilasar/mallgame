use bevy::prelude::*;
use crate::objects::components::StableObjectId;
use crate::store::chunks::StoreChunkCoord;

#[derive(Message, Debug, Clone, PartialEq)]
pub enum DomainEvent {
    ObjectBuilt {
        id: StableObjectId,
    },
    ObjectMoved {
        id: StableObjectId,
        from: Vec2,
        to: Vec2,
    },
    ObjectRotated {
        id: StableObjectId,
        from: usize,
        to: usize,
    },
    ObjectDeleted {
        id: StableObjectId,
    },
    ChunkPurchased {
        coord: StoreChunkCoord,
    },
    StoreAreaChanged {
        added_chunks: Vec<StoreChunkCoord>,
        removed_chunks: Vec<StoreChunkCoord>,
    },
}
