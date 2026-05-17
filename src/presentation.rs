use bevy::prelude::*;

use crate::components::*;
use crate::projection::{IsoProjection, world_to_iso};

pub const DEPTH_SCALE: f32 = 0.01;

pub fn sync_visual_transform_system(
    projection: Res<IsoProjection>,
    mut query: Query<(
        &WorldPos,
        &mut ProjectedPos,
        &FootAnchor,
        &VisualOffset,
        &SortLayer,
        &SortBias,
        &mut Transform,
    )>,
) {
    for (
        world_pos,
        mut projected_pos,
        foot_anchor,
        visual_offset,
        layer,
        sort_bias,
        mut transform,
    ) in &mut query
    {
        let foot_projected = world_to_iso(world_pos.0, *projection);
        projected_pos.0 = foot_projected;

        let sprite_center = foot_projected - foot_anchor.0 + visual_offset.0;
        transform.translation.x = sprite_center.x;
        transform.translation.y = sprite_center.y;
        transform.translation.z = depth_sort(foot_projected, *layer, *sort_bias);
    }
}

pub fn depth_sort(foot_projected: Vec2, layer: SortLayer, sort_bias: SortBias) -> f32 {
    layer.base_z() - foot_projected.y * DEPTH_SCALE + sort_bias.0
}

pub fn apply_selection_tint_system(
    mut query: Query<(
        &InteractionState,
        &PlacementState,
        &SelectionTint,
        &mut Sprite,
    )>,
) {
    for (interaction, placement, tint, mut sprite) in &mut query {
        sprite.color = if *placement == PlacementState::Blocked {
            tint.blocked
        } else if *placement == PlacementState::Dragging {
            tint.dragging
        } else if interaction.selected {
            tint.selected
        } else {
            tint.normal
        };
    }
}
