use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::*;
use crate::placement::footprints_overlap;
use crate::projection::{IsoProjection, cursor_to_world, world_to_iso};

#[derive(Resource, Debug, Default)]
pub struct DragState {
    pub entity: Option<Entity>,
    pub grab_offset_world: Vec2,
    pub original_world_pos: Vec2,
}

type CameraQuery<'w, 's> =
    Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera2d>>;

pub fn select_and_begin_drag_system(
    buttons: Res<ButtonInput<MouseButton>>,
    projection: Res<IsoProjection>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: CameraQuery,
    mut drag: ResMut<DragState>,
    mut query: Query<
        (
            Entity,
            &mut InteractionState,
            &mut PlacementState,
            &mut WorldPos,
            &FootAnchor,
            &Sprite,
            &Transform,
            &SortLayer,
        ),
        (With<Selectable>, With<Draggable>),
    >,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_world) = camera_cursor_world(&window_query, &camera_query, *projection) else {
        return;
    };

    let mut hit: Option<(Entity, f32, Vec2, Vec2)> = None;
    for (entity, _, _, world_pos, foot_anchor, sprite, transform, layer) in &query {
        let Some(size) = sprite.custom_size else {
            continue;
        };

        let foot_projected = world_to_iso(world_pos.0, *projection);
        let cursor_projected = world_to_iso(cursor_world, *projection);
        let sprite_center = foot_projected - foot_anchor.0;
        let local = cursor_projected - sprite_center;
        let half = size * 0.5;

        if local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y {
            let rank = layer.base_z() + transform.translation.z;
            if hit.map_or(true, |(_, best_rank, _, _)| rank > best_rank) {
                hit = Some((entity, rank, cursor_world - world_pos.0, world_pos.0));
            }
        }
    }

    for (_, mut interaction, mut placement, _, _, _, _, _) in &mut query {
        interaction.selected = false;
        interaction.hovered = false;
        if *placement == PlacementState::Dragging {
            *placement = PlacementState::Placed;
        }
    }

    if let Some((entity, _, grab_offset_world, original_world_pos)) = hit {
        if let Ok((_, mut interaction, mut placement, _, _, _, _, _)) = query.get_mut(entity) {
            interaction.selected = true;
            *placement = PlacementState::Dragging;
            drag.entity = Some(entity);
            drag.grab_offset_world = grab_offset_world;
            drag.original_world_pos = original_world_pos;
        }
    } else {
        drag.entity = None;
        drag.grab_offset_world = Vec2::ZERO;
        drag.original_world_pos = Vec2::ZERO;
    }
}

pub fn drag_system(
    buttons: Res<ButtonInput<MouseButton>>,
    projection: Res<IsoProjection>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: CameraQuery,
    drag: Res<DragState>,
    mut queries: ParamSet<(
        Query<(Entity, &WorldPos, &CollisionFootprint), With<BlocksPlacement>>,
        Query<(&mut WorldPos, &mut PlacementState), With<Draggable>>,
    )>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let Some(entity) = drag.entity else {
        return;
    };
    let Some(cursor_world) = camera_cursor_world(&window_query, &camera_query, *projection) else {
        return;
    };

    let target_world_pos = cursor_world - drag.grab_offset_world;

    let Some(footprint) = queries
        .p0()
        .iter()
        .find_map(|(candidate, _, footprint)| (candidate == entity).then_some(*footprint))
    else {
        return;
    };

    let blocked = queries
        .p0()
        .iter()
        .filter(|(other_entity, _, _)| *other_entity != entity)
        .any(|(_, other_pos, other_footprint)| {
            footprints_overlap(target_world_pos, footprint, other_pos.0, *other_footprint)
        });

    let mut draggable = queries.p1();
    let Ok((mut world_pos, mut placement)) = draggable.get_mut(entity) else {
        return;
    };

    world_pos.0 = target_world_pos;
    *placement = if blocked {
        PlacementState::Blocked
    } else {
        PlacementState::Dragging
    };
}

pub fn end_drag_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    mut query: Query<(&mut WorldPos, &mut PlacementState)>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    if let Some(entity) = drag.entity.take() {
        if let Ok((mut world_pos, mut placement)) = query.get_mut(entity) {
            if *placement == PlacementState::Blocked {
                world_pos.0 = drag.original_world_pos;
            }
            *placement = PlacementState::Placed;
        }
    }
    drag.grab_offset_world = Vec2::ZERO;
    drag.original_world_pos = Vec2::ZERO;
}

pub fn print_positions_system(
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<(&PlaceableAssetId, &WorldPos, &SortLayer, &FootAnchor)>,
) {
    if !keys.just_pressed(KeyCode::KeyP) {
        return;
    }

    info!("--- placeable positions ---");
    for (asset_id, world_pos, sort_layer, foot_anchor) in &query {
        info!(
            "asset_id={} world_x={:.2} world_y={:.2} sort_layer={:?} foot_anchor=({:.2},{:.2})",
            asset_id.0, world_pos.0.x, world_pos.0.y, sort_layer, foot_anchor.0.x, foot_anchor.0.y
        );
    }
}

pub fn camera_cursor_world(
    window_query: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &CameraQuery,
    projection: IsoProjection,
) -> Option<Vec2> {
    let window = window_query.iter().next()?;
    let (camera, camera_transform) = camera_query.iter().next()?;
    cursor_to_world(window, camera, camera_transform, projection)
}
