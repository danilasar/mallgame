use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::tools::ToolChangedRequested;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolMode {
    #[default]
    Cursor,
    Move,
    Delete,
    Build,
}

#[derive(Debug, Clone)]
pub struct ToolDescriptor {
    pub mode: ToolMode,
    pub action: InputAction,
    pub label: &'static str,
}

#[derive(Resource, Debug, Default)]
pub struct ToolRegistry {
    pub tools: Vec<ToolDescriptor>,
}

impl ToolRegistry {
    pub fn register(&mut self, descriptor: ToolDescriptor) {
        self.tools.retain(|tool| tool.mode != descriptor.mode);
        self.tools.push(descriptor);
    }
}

pub fn tool_hotkeys_system(
    actions: Res<InputActionState>,
    registry: Res<ToolRegistry>,
    mut next: ResMut<NextState<ToolMode>>,
    mut changed: MessageWriter<ToolChangedRequested>,
) {
    for tool in &registry.tools {
        if actions.just_pressed(tool.action) {
            info!("Tool hotkey requested: {}", tool.label);
            changed.write(ToolChangedRequested { mode: tool.mode });
            next.set(tool.mode);
            break;
        }
    }
}

pub fn log_tool_changed_requests(mut changed: MessageReader<ToolChangedRequested>) {
    for request in changed.read() {
        info!("ToolChangedRequested mode={:?}", request.mode);
    }
}
