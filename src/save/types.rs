use serde::{Deserialize, Serialize};
use crate::store::{StoreChunkCoord, StoreChunkKind};
use crate::objects::components::StableObjectId;
use crate::objects::prototypes::BuildPrototypeId;

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
    pub prototype_id: BuildPrototypeId,
    pub world_pos: WorldPosSave,
    pub rotation_index: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldPosSave {
    pub x: f32,
    pub y: f32,
}
