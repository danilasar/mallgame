use bevy::prelude::*;
use crate::objects::components::*;
use crate::presentation::{world_to_iso, IsoProjection};

pub fn sync_npc_visual_transform(
    projection: Res<IsoProjection>,
    mut query: Query<(
        &WorldPos,
        &VisualOffset,
        &SortLayer,
        &mut Transform,
    )>,
) {
    for (world_pos, visual_offset, sort_layer, mut transform) in query.iter_mut() {
        let projected = world_to_iso(world_pos.0, *projection);
        let z = sort_layer.base_z() + (projected.y * 0.001); // Simple depth sort
        
        transform.translation = Vec3::new(
            projected.x + visual_offset.0.x,
            projected.y + visual_offset.0.y,
            z,
        );
    }
}
