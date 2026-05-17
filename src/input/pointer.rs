use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::presentation::{IsoProjection, iso_to_world};

#[derive(Resource, Debug, Clone)]
pub struct PointerContext {
    pub screen_pos: Vec2,
    pub projected_pos: Vec2,
    pub world_pos: Vec2,
    pub hovered_entity: Option<Entity>,
    pub over_ui: bool,
    pub has_pointer: bool,
}

impl Default for PointerContext {
    fn default() -> Self {
        Self {
            screen_pos: Vec2::ZERO,
            projected_pos: Vec2::ZERO,
            world_pos: Vec2::ZERO,
            hovered_entity: None,
            over_ui: false,
            has_pointer: false,
        }
    }
}

pub fn update_pointer_context(
    projection: Res<IsoProjection>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut pointer: ResMut<PointerContext>,
) {
    let Some(window) = window_query.iter().next() else {
        pointer.has_pointer = false;
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        pointer.has_pointer = false;
        return;
    };
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        pointer.has_pointer = false;
        return;
    };
    let Ok(projected_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else {
        pointer.has_pointer = false;
        return;
    };

    pointer.screen_pos = screen_pos;
    pointer.projected_pos = projected_pos;
    pointer.world_pos = iso_to_world(projected_pos, *projection);
    pointer.over_ui = false;
    pointer.has_pointer = true;
}
