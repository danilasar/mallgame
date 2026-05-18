use crate::objects::components::StableObjectId;
use crate::objects::prototypes::BuildObjectId;
use crate::store::{StoreBoundarySide, StoreChunkCoord, StoreChunkKind};
use serde::{Deserialize, Serialize};

pub type SaveVersion = u32;
pub const CURRENT_SAVE_VERSION: SaveVersion = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveGame {
    pub version: SaveVersion,
    pub next_object_id: u64,
    pub store: StoreSave,
    pub objects: Vec<ObjectSave>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreSave {
    pub owned_chunks: Vec<StoreChunkSave>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreChunkSave {
    pub coord: StoreChunkCoord,
    pub kind: StoreChunkKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectSave {
    pub id: StableObjectId,
    pub prototype_id: BuildObjectId,
    pub placement: ObjectPlacementSave,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldPosSave {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ObjectPlacementSave {
    Floor {
        world_pos: WorldPosSave,
        rotation_index: Option<usize>,
    },
    WallMounted {
        segment_key: WallSegmentKeySave,
        offset_along_segment: f32,
        height_on_wall: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WallSegmentKeySave {
    pub chunk: StoreChunkCoord,
    pub side: StoreBoundarySide,
}
