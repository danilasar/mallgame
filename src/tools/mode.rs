use bevy::prelude::*;

use crate::tools::ToolChangedRequested;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolMode {
    #[default]
    Cursor,
    Move,
    Delete,
    Build,
}

pub fn tool_hotkeys_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<ToolMode>>,
    mut changed: MessageWriter<ToolChangedRequested>,
) {
    let requested = if keys.just_pressed(KeyCode::Digit1) {
        Some(ToolMode::Cursor)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(ToolMode::Move)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(ToolMode::Delete)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(ToolMode::Build)
    } else {
        None
    };

    if let Some(mode) = requested {
        changed.write(ToolChangedRequested { mode });
        next.set(mode);
    }
}

pub fn log_tool_changed_requests(mut changed: MessageReader<ToolChangedRequested>) {
    for request in changed.read() {
        info!("ToolChangedRequested mode={:?}", request.mode);
    }
}
