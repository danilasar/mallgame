use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    ToolCursor,
    ToolMove,
    ToolDelete,
    ToolBuild,
    PrimaryClick,
    SecondaryClick,
    Cancel,
    Confirm,
    Rotate,
    QuickSave,
    QuickLoad,
    CameraZoomIn,
    CameraZoomOut,
    ToggleFullscreen,
    PrintDebugPositions,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct InputActionState {
    pub pressed: HashSet<InputAction>,
    pub just_pressed: HashSet<InputAction>,
    pub just_released: HashSet<InputAction>,
}

impl InputActionState {
    pub fn pressed(&self, action: InputAction) -> bool {
        self.pressed.contains(&action)
    }
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }
    pub fn just_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }
}

pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

#[derive(Resource)]
pub struct InputBindings {
    pub map: HashMap<InputAction, Vec<InputBinding>>,
}

impl Default for InputBindings {
    fn default() -> Self {
        let mut bindings = Self {
            map: HashMap::new(),
        };
        bindings.insert(InputAction::ToolCursor, InputBinding::Key(KeyCode::Digit1));
        bindings.insert(InputAction::ToolMove, InputBinding::Key(KeyCode::Digit2));
        bindings.insert(InputAction::ToolDelete, InputBinding::Key(KeyCode::Digit3));
        bindings.insert(InputAction::ToolBuild, InputBinding::Key(KeyCode::Digit4));
        bindings.insert(
            InputAction::PrimaryClick,
            InputBinding::Mouse(MouseButton::Left),
        );
        bindings.insert(
            InputAction::SecondaryClick,
            InputBinding::Mouse(MouseButton::Right),
        );
        bindings.insert(InputAction::Cancel, InputBinding::Key(KeyCode::Escape));
        bindings.insert(InputAction::Confirm, InputBinding::Key(KeyCode::Enter));
        bindings.insert(InputAction::Rotate, InputBinding::Key(KeyCode::KeyR));
        bindings.insert(InputAction::QuickSave, InputBinding::Key(KeyCode::F5));
        bindings.insert(InputAction::QuickLoad, InputBinding::Key(KeyCode::F8));
        bindings.insert(InputAction::CameraZoomIn, InputBinding::Key(KeyCode::Equal));
        bindings.insert(
            InputAction::CameraZoomOut,
            InputBinding::Key(KeyCode::Minus),
        );
        bindings.insert(
            InputAction::ToggleFullscreen,
            InputBinding::Key(KeyCode::F11),
        );
        bindings.insert(
            InputAction::PrintDebugPositions,
            InputBinding::Key(KeyCode::F1),
        );
        bindings
    }
}

impl InputBindings {
    pub fn insert(&mut self, action: InputAction, binding: InputBinding) {
        self.map.entry(action).or_default().push(binding);
    }

    pub fn just_pressed(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.map.get(&action).is_some_and(|bindings| {
            bindings.iter().any(|b| match b {
                InputBinding::Key(k) => keys.just_pressed(*k),
                InputBinding::Mouse(m) => mouse.just_pressed(*m),
            })
        })
    }

    pub fn pressed(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.map.get(&action).is_some_and(|bindings| {
            bindings.iter().any(|b| match b {
                InputBinding::Key(k) => keys.pressed(*k),
                InputBinding::Mouse(m) => mouse.pressed(*m),
            })
        })
    }

    pub fn just_released(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.map.get(&action).is_some_and(|bindings| {
            bindings.iter().any(|b| match b {
                InputBinding::Key(k) => keys.just_released(*k),
                InputBinding::Mouse(m) => mouse.just_released(*m),
            })
        })
    }
}

pub struct InputActionsPlugin;

impl Plugin for InputActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActionState>()
            .init_resource::<InputBindings>()
            .add_systems(PreUpdate, update_input_action_state);
    }
}

pub fn update_input_action_state(
    bindings: Res<InputBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<InputActionState>,
) {
    state.pressed.clear();
    state.just_pressed.clear();
    state.just_released.clear();

    for &action in bindings.map.keys() {
        if bindings.pressed(action, &keys, &mouse) {
            state.pressed.insert(action);
        }
        if bindings.just_pressed(action, &keys, &mouse) {
            state.just_pressed.insert(action);
        }
        if bindings.just_released(action, &keys, &mouse) {
            state.just_released.insert(action);
        }
    }
}
