use crate::objects::components::{
    ObjectPlacement, ObjectPlacementComponent, ObjectPrototypeId, ObjectStableId,
    StableObjectIdAllocator, StoreObject,
};
use crate::save::types::*;
use crate::store::StoreArea;
use crate::tools::ToolPreview;
use bevy::prelude::*;

type SaveObjectsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static ObjectStableId,
        &'static ObjectPrototypeId,
        &'static ObjectPlacementComponent,
    ),
    (With<StoreObject>, Without<ToolPreview>),
>;

#[allow(clippy::type_complexity)]
pub fn extract_save_game(
    store: &StoreArea,
    allocator: &StableObjectIdAllocator,
    objects_query: &SaveObjectsQuery<'_, '_>,
) -> SaveGame {
    let mut saved_objects: Vec<ObjectSave> = objects_query
        .iter()
        .map(|(stable_id, proto_id, placement)| ObjectSave {
            id: stable_id.0,
            prototype_id: proto_id.0.clone(),
            placement: placement_save(placement.placement),
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

fn placement_save(placement: ObjectPlacement) -> ObjectPlacementSave {
    match placement {
        ObjectPlacement::Floor {
            world_pos,
            rotation_index,
        } => ObjectPlacementSave::Floor {
            world_pos: WorldPosSave {
                x: world_pos.x,
                y: world_pos.y,
            },
            rotation_index,
        },
        ObjectPlacement::WallMounted { attachment } => ObjectPlacementSave::WallMounted {
            segment_key: WallSegmentKeySave {
                chunk: attachment.segment_key.chunk,
                side: attachment.segment_key.side,
            },
            offset_along_segment: attachment.offset_along_segment,
            height_on_wall: attachment.height_on_wall,
        },
    }
}
