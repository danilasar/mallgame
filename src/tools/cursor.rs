use bevy::prelude::*;

use crate::input::{InputAction, PointerContext, PointerTargets};
use crate::objects::components::Interactive;
use crate::tools::{
    ObjectActionKind, ObjectActionOrigin, ObjectActionRequested, SelectionState, ToolContext,
    ToolDescriptor, ToolInputGate, ToolMode, ToolRegistry, ToolSet,
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
    targets: Res<PointerTargets>,
    gate: Res<ToolInputGate>,
    interactive: Query<(), With<Interactive>>,
    selection: Res<SelectionState>,
    mut tool: ResMut<ToolContext>,
    mut actions: MessageWriter<ObjectActionRequested>,
) {
    tool.sync_from_pointer(&pointer, &targets);

    if !gate.can_use_world() {
        return;
    }

    if gate.primary_world_click_released {
        if let Some(entity) = tool
            .hovered_entity
            .filter(|entity| interactive.get(*entity).is_ok())
        {
            actions.write(ObjectActionRequested {
                entity,
                action: ObjectActionKind::Inspect,
                origin: ObjectActionOrigin::CursorClick,
            });
        } else if let Some(primary) = selection.primary {
            // Click on empty world clears selection
            actions.write(ObjectActionRequested {
                entity: primary,
                action: ObjectActionKind::Deselect,
                origin: ObjectActionOrigin::CursorClick,
            });
        }
    }
}
