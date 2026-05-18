use crate::input::InputAction;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum ToolMode {
    #[default]
    Cursor,
    Move,
    #[allow(dead_code)]
    Rotate,
    Delete,
    Build,
    Expansion,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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
        if !self.tools.iter().any(|t| t.mode == descriptor.mode) {
            self.tools.push(descriptor);
        }
    }
}

#[allow(dead_code)]
pub fn tool_hotkeys_system(
    actions: Res<crate::input::InputActionState>,
    mut next_mode: ResMut<NextState<ToolMode>>,
) {
    if actions.just_pressed(InputAction::ToolCursor) {
        next_mode.set(ToolMode::Cursor);
    }
    if actions.just_pressed(InputAction::ToolBuild) {
        next_mode.set(ToolMode::Build);
    }
}

pub fn log_tool_changed_requests(mut events: MessageReader<crate::tools::ToolChangedRequested>) {
    for event in events.read() {
        info!("ToolChangedRequested mode={:?}", event.mode);
    }
}
