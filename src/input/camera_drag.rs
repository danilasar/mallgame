use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext};
use crate::ui::ModalState;

#[derive(Resource, Debug, Clone)]
pub struct PointerDragState {
    pub pressed_at_screen: Option<Vec2>,
    pub previous_screen_pos: Option<Vec2>,
    pub is_camera_dragging: bool,
    pub consumed_click: bool,
    pub drag_threshold_px: f32,
}

impl Default for PointerDragState {
    fn default() -> Self {
        Self {
            pressed_at_screen: None,
            previous_screen_pos: None,
            is_camera_dragging: false,
            consumed_click: false,
            drag_threshold_px: 6.0,
        }
    }
}

pub fn camera_drag_system(
    actions: Res<InputActionState>,
    pointer: Res<PointerContext>,
    modal: Res<ModalState>,
    mut drag: ResMut<PointerDragState>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    drag.consumed_click = false;

    if modal.active.is_some() || pointer.over_ui || !pointer.has_pointer {
        return;
    }

    if actions.just_pressed(InputAction::PrimaryClick) {
        drag.pressed_at_screen = Some(pointer.screen_pos);
        drag.previous_screen_pos = Some(pointer.screen_pos);
        drag.is_camera_dragging = false;
    }

    if actions.pressed(InputAction::PrimaryClick) {
        if let (Some(start), Some(previous)) = (drag.pressed_at_screen, drag.previous_screen_pos) {
            if pointer.screen_pos.distance(start) > drag.drag_threshold_px {
                drag.is_camera_dragging = true;
            }

            if drag.is_camera_dragging {
                let delta = pointer.screen_pos - previous;
                if let Some(mut camera_transform) = camera_query.iter_mut().next() {
                    camera_transform.translation.x -= delta.x;
                    camera_transform.translation.y += delta.y;
                }
            }
        }
        drag.previous_screen_pos = Some(pointer.screen_pos);
    }

    if actions.just_released(InputAction::PrimaryClick) {
        drag.consumed_click = drag.is_camera_dragging;
        drag.pressed_at_screen = None;
        drag.previous_screen_pos = None;
        drag.is_camera_dragging = false;
    }
}
