use bevy::prelude::*;

use crate::input::{InputAction, PointerContext};
use crate::objects::components::Interactive;
use crate::tools::{
    ObjectAction, ObjectActionRequested, ToolContext, ToolDescriptor, ToolInputGate, ToolMode,
    ToolRegistry, ToolSet,
};

pub struct CursorToolPlugin;

impl Plugin for CursorToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Cursor,
                action: InputAction::ToolCursor,
                label: "Cursor",
            });

        app.add_systems(
            Update,
            cursor_tool_system
                .run_if(in_state(ToolMode::Cursor))
                .in_set(ToolSet::ToolUpdate),
        );
    }
}

pub fn cursor_tool_system(
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    interactive: Query<(), With<Interactive>>,
    mut tool: ResMut<ToolContext>,
    mut actions: MessageWriter<ObjectActionRequested>,
) {
    tool.sync_from_pointer(&pointer);
    tool.active = None;

    if !gate.can_use_world() {
        return;
    }

    if gate.primary_click_released {
        if let Some(entity) = tool
            .hovered
            .filter(|entity| interactive.get(*entity).is_ok())
        {
            actions.write(ObjectActionRequested {
                entity,
                action: ObjectAction::Select,
            });
        }
    }
}
