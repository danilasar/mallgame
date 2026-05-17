use bevy::prelude::*;

use crate::input::{InputAction, PointerContext};
use crate::objects::components::Deletable;
use crate::tools::{
    ActiveToolAction, ToolContext, ToolDescriptor, ToolInputGate, ToolMode, ToolRegistry, ToolSet,
};
use crate::ui::{Modal, ModalState};

pub struct DeleteToolPlugin;

impl Plugin for DeleteToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Delete,
                action: InputAction::ToolDelete,
                label: "Delete",
            });

        app.add_systems(
            Update,
            delete_tool_system
                .run_if(in_state(ToolMode::Delete))
                .in_set(ToolSet::ToolUpdate),
        );
    }
}

pub fn delete_tool_system(
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    deletable: Query<(), With<Deletable>>,
    mut tool: ResMut<ToolContext>,
    mut modal_state: ResMut<ModalState>,
) {
    tool.sync_from_pointer(&pointer);

    if !gate.can_use_world() {
        return;
    }

    if gate.primary_click_released {
        if let Some(entity) = tool.hovered.filter(|entity| deletable.get(*entity).is_ok()) {
            tool.active = Some(ActiveToolAction::PendingDelete { entity });
            open_confirm_delete_modal(&mut modal_state, entity);
        }
    }
}

pub fn open_confirm_delete_modal(modal_state: &mut ModalState, entity: Entity) {
    modal_state.active = Some(Modal::ConfirmDelete { entity });
    info!(
        "Confirm delete modal opened for entity={:?}. Enter=confirm, Escape/right click=cancel",
        entity
    );
}
