use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerDragState};
use crate::ui::ModalStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerPressOwner {
    #[default]
    None,
    World,
    Ui,
    WorldWidget,
    Modal,
    CameraDrag,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PrimaryPointerCycle {
    pub owner: PointerPressOwner,
    pub consumed: bool,
    pub drag_started: bool,
    pub started_this_frame: bool,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ToolInputGate {
    pub world_blocked: bool,
    pub pointer_available: bool,

    pub primary_world_press_started: bool,
    pub primary_world_click_released: bool,

    pub cancel_requested: bool,
    pub confirm_requested: bool,

    pub blocked_by_ui: bool,
    pub blocked_by_modal: bool,
}

impl ToolInputGate {
    pub fn can_use_world(&self) -> bool {
        !self.world_blocked && self.pointer_available
    }
}

pub fn update_tool_input_gate(
    actions: Res<InputActionState>,
    pointer: Res<PointerContext>,
    drag: Res<PointerDragState>,
    modal: Res<ModalStack>,
    mut cycle: ResMut<PrimaryPointerCycle>,
    mut gate: ResMut<ToolInputGate>,
) {
    gate.pointer_available = pointer.has_pointer;

    let is_modal_blocking = modal.stack.last().is_some_and(|m| m.blocks_world);
    let is_ui_blocking = pointer.over_ui;

    gate.blocked_by_modal = is_modal_blocking;
    gate.blocked_by_ui = is_ui_blocking;
    gate.world_blocked = is_modal_blocking || is_ui_blocking || !pointer.has_pointer;

    // Manage Cycle
    if actions.just_pressed(InputAction::PrimaryClick) {
        cycle.started_this_frame = true;
        cycle.consumed = false;
        cycle.drag_started = false;

        // Only set if not already overridden by a system earlier in the frame (like a widget)
        if cycle.owner == PointerPressOwner::None {
            if is_modal_blocking {
                cycle.owner = PointerPressOwner::Modal;
            } else if is_ui_blocking {
                cycle.owner = PointerPressOwner::Ui;
            } else if pointer.has_pointer {
                cycle.owner = PointerPressOwner::World;
            } else {
                cycle.owner = PointerPressOwner::None;
            }
        }
    } else {
        cycle.started_this_frame = false;
    }

    if drag.is_camera_dragging {
        cycle.drag_started = true;
    }
    if drag.consumed_click {
        cycle.consumed = true;
    }

    // Derive Signals
    gate.primary_world_press_started =
        cycle.started_this_frame && cycle.owner == PointerPressOwner::World;

    gate.primary_world_click_released = actions.just_released(InputAction::PrimaryClick)
        && cycle.owner == PointerPressOwner::World
        && !cycle.consumed
        && !cycle.drag_started
        && !gate.world_blocked;

    // Reset owner ONLY after release is processed by signals this frame
    if actions.just_released(InputAction::PrimaryClick) {
        cycle.owner = PointerPressOwner::None;
    }

    gate.cancel_requested = actions.just_pressed(InputAction::Cancel);
    gate.confirm_requested = actions.just_pressed(InputAction::Confirm);
}
