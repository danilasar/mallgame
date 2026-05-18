use crate::objects::components::StableObjectId;
use crate::objects::prototypes::BuildObjectId;
use crate::save::types::*;
use crate::store::{StoreChunkCoord, WorldBounds};
use bevy::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LoadIssue {
    UnknownPrototype(BuildObjectId),
    InvalidRotationIndex {
        object_id: StableObjectId,
        rotation_index: usize,
    },
    DuplicateStableObjectId(StableObjectId),
    InvalidChunk(StoreChunkCoord),
    ObjectOutsideStoreArea {
        object_id: StableObjectId,
    },
    ObjectPlacementInvalid {
        object_id: StableObjectId,
    },
    AllocatorNextIdTooSmall {
        save_next: u64,
        normalized_next: u64,
    },
    NonFiniteWorldPos {
        object_id: StableObjectId,
    },
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LoadReport {
    pub loaded_objects: usize,
    pub skipped_objects: usize,
    pub loaded_chunks: usize,
    pub issues: Vec<LoadIssue>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SaveLoadError {
    UnsupportedVersion(u32),
    FatalValidationError(Vec<LoadIssue>),
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct SaveLoadLimits {
    pub max_objects: usize,
    pub max_chunks: usize,
}

impl Default for SaveLoadLimits {
    fn default() -> Self {
        Self {
            max_objects: 1000,
            max_chunks: 400,
        }
    }
}

pub struct StoreAreaValidationReport {
    pub valid_chunks: Vec<StoreChunkSave>,
    pub issues: Vec<LoadIssue>,
    pub fatal: bool,
}

pub fn validate_loaded_store_area(
    store_save: &StoreSave,
    _world_bounds: &WorldBounds,
) -> StoreAreaValidationReport {
    let mut valid_chunks = Vec::new();
    let mut issues = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for chunk in &store_save.owned_chunks {
        if !seen.insert(chunk.coord) {
            // Duplicate chunk in save is not necessarily fatal but suspicious
            continue;
        }

        // Basic bounds check
        if chunk.coord.x.abs() > 100 || chunk.coord.y.abs() > 100 {
            issues.push(LoadIssue::InvalidChunk(chunk.coord));
            continue;
        }

        valid_chunks.push(chunk.clone());
    }

    StoreAreaValidationReport {
        valid_chunks,
        issues,
        fatal: false,
    }
}
