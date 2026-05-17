use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::tools::{DeleteObjectRequested, ToolContext};

#[derive(Resource, Debug, Default)]
pub struct ModalState {
    pub active: Option<Modal>,
}

#[derive(Debug, Clone, Copy)]
pub enum Modal {
    ConfirmDelete { entity: Entity },
}

pub fn modal_input_system(
    actions: Res<InputActionState>,
    mut modal: ResMut<ModalState>,
    mut tool: ResMut<ToolContext>,
    mut deletes: MessageWriter<DeleteObjectRequested>,
) {
    let Some(active) = modal.active else {
        return;
    };

    if actions.just_pressed(InputAction::Confirm) {
        confirm_delete(active, &mut modal, &mut tool, &mut deletes);
    } else if actions.just_pressed(InputAction::Cancel) {
        cancel_modal(&mut modal, &mut tool);
    }
}

pub fn confirm_delete(
    modal: Modal,
    modal_state: &mut ModalState,
    tool: &mut ToolContext,
    deletes: &mut MessageWriter<DeleteObjectRequested>,
) {
    match modal {
        Modal::ConfirmDelete { entity } => {
            deletes.write(DeleteObjectRequested { entity });
        }
    }
    modal_state.active = None;
    tool.active = None;
}

pub fn cancel_modal(modal_state: &mut ModalState, tool: &mut ToolContext) {
    modal_state.active = None;
    tool.active = None;
}
