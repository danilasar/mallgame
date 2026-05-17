use bevy::prelude::*;

use crate::input::{InputAction, PointerContext, PointerTargets};
use crate::objects::components::{Deletable, StoreObject};
use crate::tools::{
    ObjectActionKind, ObjectActionOrigin, ObjectActionRequested, ToolContext, ToolDescriptor,
    ToolInputGate, ToolMode, ToolRegistry, ToolSet,
};

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
    targets: Res<PointerTargets>,
    gate: Res<ToolInputGate>,
    deletable: Query<(), (With<Deletable>, With<StoreObject>)>,
    mut tool: ResMut<ToolContext>,
    mut actions: MessageWriter<ObjectActionRequested>,
) {
    tool.sync_from_pointer(&pointer, &targets);

    if !gate.can_use_world() {
        return;
    }

    if gate.primary_world_click_released {
        if let Some(entity) = tool.hovered_object.filter(|entity| deletable.get(*entity).is_ok()) {
            actions.write(ObjectActionRequested {
                entity,
                action: ObjectActionKind::Delete,
                origin: ObjectActionOrigin::CursorClick,
            });
        }
    }
}
