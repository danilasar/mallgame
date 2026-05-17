use bevy::prelude::*;

use crate::input::{PointerContext, PointerDragState};
use crate::ui::ModalState;

#[derive(Resource, Debug, Clone, Default)]
pub struct ToolInputGate {
    pub world_blocked: bool,
    pub pointer_available: bool,
    pub primary_click_released: bool,
    pub cancel_requested: bool,
}

impl ToolInputGate {
    pub fn can_use_world(&self) -> bool {
        !self.world_blocked && self.pointer_available
    }
}

pub fn update_tool_input_gate(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<PointerContext>,
    drag: Res<PointerDragState>,
    modal: Res<ModalState>,
    mut gate: ResMut<ToolInputGate>,
) {
    gate.pointer_available = pointer.has_pointer;
    gate.world_blocked = modal.active.is_some() || pointer.over_ui || !pointer.has_pointer;
    gate.primary_click_released =
        buttons.just_released(MouseButton::Left) && !drag.consumed_click && !gate.world_blocked;
    gate.cancel_requested = (keys.just_pressed(KeyCode::Escape)
        || buttons.just_pressed(MouseButton::Right))
        && !modal.active.is_some();
}
