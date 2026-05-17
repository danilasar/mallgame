use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext};
use crate::tools::{PointerPressOwner, PrimaryPointerCycle};
use crate::ui::ModalStack;

#[derive(Resource, Debug, Clone)]
pub struct PointerDragState {
    pub pressed_at_screen: Option<Vec2>,
    pub grabbed_projected_pos: Option<Vec2>,
    pub is_camera_dragging: bool,
    pub consumed_click: bool,
    pub drag_threshold_px: f32,
}

impl Default for PointerDragState {
    fn default() -> Self {
        Self {
            pressed_at_screen: None,
            grabbed_projected_pos: None,
            is_camera_dragging: false,
            consumed_click: false,
            drag_threshold_px: 6.0,
        }
    }
}

pub fn camera_drag_system(
    actions: Res<InputActionState>,
    pointer: Res<PointerContext>,
    modal: Res<ModalStack>,
    mut drag: ResMut<PointerDragState>,
    mut cycle: ResMut<PrimaryPointerCycle>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    drag.consumed_click = false;

    if !modal.stack.is_empty() || pointer.over_ui || !pointer.has_pointer {
        return;
    }

    if actions.just_pressed(InputAction::PrimaryClick) {
        drag.pressed_at_screen = Some(pointer.screen_pos);
        drag.grabbed_projected_pos = Some(pointer.projected_pos);
        drag.is_camera_dragging = false;
    }

    if actions.pressed(InputAction::PrimaryClick) {
        if let (Some(start), Some(grabbed_projected)) =
            (drag.pressed_at_screen, drag.grabbed_projected_pos)
        {
            if pointer.screen_pos.distance(start) > drag.drag_threshold_px {
                drag.is_camera_dragging = true;
                cycle.owner = PointerPressOwner::CameraDrag;
            }

            if drag.is_camera_dragging {
                let delta = grabbed_projected - pointer.projected_pos;
                if let Some(mut camera_transform) = camera_query.iter_mut().next() {
                    camera_transform.translation.x += delta.x;
                    camera_transform.translation.y += delta.y;
                }
            }
        }
    }

    if actions.just_released(InputAction::PrimaryClick) {
        drag.consumed_click = drag.is_camera_dragging;
        drag.pressed_at_screen = None;
        drag.grabbed_projected_pos = None;
        drag.is_camera_dragging = false;
    }
}
