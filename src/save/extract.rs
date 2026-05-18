use crate::objects::components::{
    ObjectPrototypeId, ObjectStableId, StableObjectIdAllocator, StoreObject, WorldPos,
};
use crate::objects::rotation::Rotatable;
use crate::save::types::*;
use crate::store::StoreArea;
use crate::tools::ToolPreview;
use bevy::prelude::*;

pub fn extract_save_game(
    store: &StoreArea,
    allocator: &StableObjectIdAllocator,
    objects_query: &Query<
        (
            &ObjectStableId,
            &ObjectPrototypeId,
            &WorldPos,
            Option<&Rotatable>,
        ),
        (With<StoreObject>, Without<ToolPreview>),
    >,
) -> SaveGame {
    let mut saved_objects: Vec<ObjectSave> = objects_query
        .iter()
        .map(|(stable_id, proto_id, pos, rotatable)| ObjectSave {
            id: stable_id.0,
            prototype_id: proto_id.0.clone(),
            world_pos: WorldPosSave {
                x: pos.0.x,
                y: pos.0.y,
            },
            rotation_index: rotatable.map(|r| r.current),
        })
        .collect();

    // Deterministic sort
    saved_objects.sort_by_key(|o| o.id.0);

    let mut saved_chunks: Vec<StoreChunkSave> = store
        .owned_chunks
        .iter()
        .map(|(coord, data)| StoreChunkSave {
            coord: *coord,
            kind: data.kind,
        })
        .collect();

    // Deterministic sort
    saved_chunks.sort_by_key(|c| (c.coord.y, c.coord.x));

    SaveGame {
        version: CURRENT_SAVE_VERSION,
        next_object_id: allocator.next,
        store: StoreSave {
            owned_chunks: saved_chunks,
        },
        objects: saved_objects,
    }
}
