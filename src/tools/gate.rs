use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerDragState};
use crate::ui::ModalState;

#[derive(Resource, Debug, Clone, Default)]
pub struct ToolInputGate {
    pub world_blocked: bool,
    pub pointer_available: bool,
    pub primary_click_pressed: bool,
    pub primary_click_released: bool,
    pub cancel_requested: bool,
    pub confirm_requested: bool,
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
    modal: Res<ModalState>,
    mut gate: ResMut<ToolInputGate>,
) {
    gate.pointer_available = pointer.has_pointer;
    gate.world_blocked = modal.active.is_some() || pointer.over_ui || !pointer.has_pointer;
    gate.primary_click_pressed = actions.just_pressed(InputAction::PrimaryClick)
        && !drag.consumed_click
        && !gate.world_blocked;
    gate.primary_click_released = actions.just_released(InputAction::PrimaryClick)
        && !drag.consumed_click
        && !gate.world_blocked;
    gate.cancel_requested = actions.just_pressed(InputAction::Cancel);
    gate.confirm_requested = actions.just_pressed(InputAction::Confirm);
}
