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
    CameraZoomIn,
    CameraZoomOut,
    ToggleFullscreen,
    PrintDebugPositions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

#[derive(Resource, Debug, Clone)]
pub struct InputBindings {
    map: HashMap<InputAction, Vec<InputBinding>>,
}

impl Default for InputBindings {
    fn default() -> Self {
        use InputAction::*;
        use InputBinding::*;

        let mut map = HashMap::new();
        map.insert(ToolCursor, vec![Key(KeyCode::Digit1)]);
        map.insert(ToolMove, vec![Key(KeyCode::Digit2)]);
        map.insert(ToolDelete, vec![Key(KeyCode::Digit3)]);
        map.insert(ToolBuild, vec![Key(KeyCode::Digit4)]);
        map.insert(PrimaryClick, vec![Mouse(MouseButton::Left)]);
        map.insert(SecondaryClick, vec![Mouse(MouseButton::Right)]);
        map.insert(
            Cancel,
            vec![Key(KeyCode::Escape), Mouse(MouseButton::Right)],
        );
        map.insert(
            Confirm,
            vec![Key(KeyCode::Enter), Key(KeyCode::NumpadEnter)],
        );
        map.insert(
            CameraZoomIn,
            vec![Key(KeyCode::Equal), Key(KeyCode::NumpadAdd)],
        );
        map.insert(
            CameraZoomOut,
            vec![Key(KeyCode::Minus), Key(KeyCode::NumpadSubtract)],
        );
        map.insert(ToggleFullscreen, vec![Key(KeyCode::F11)]);
        map.insert(PrintDebugPositions, vec![Key(KeyCode::KeyP)]);

        Self { map }
    }
}

impl InputBindings {
    pub fn actions(&self) -> impl Iterator<Item = InputAction> + '_ {
        self.map.keys().copied()
    }

    pub fn pressed(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.any_binding(action, |binding| match binding {
            InputBinding::Key(key) => keys.pressed(key),
            InputBinding::Mouse(button) => mouse.pressed(button),
        })
    }

    pub fn just_pressed(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.any_binding(action, |binding| match binding {
            InputBinding::Key(key) => keys.just_pressed(key),
            InputBinding::Mouse(button) => mouse.just_pressed(button),
        })
    }

    pub fn just_released(
        &self,
        action: InputAction,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.any_binding(action, |binding| match binding {
            InputBinding::Key(key) => keys.just_released(key),
            InputBinding::Mouse(button) => mouse.just_released(button),
        })
    }

    fn any_binding(&self, action: InputAction, predicate: impl Fn(InputBinding) -> bool) -> bool {
        self.map
            .get(&action)
            .is_some_and(|bindings| bindings.iter().copied().any(predicate))
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct InputActionState {
    pressed: HashSet<InputAction>,
    just_pressed: HashSet<InputAction>,
    just_released: HashSet<InputAction>,
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

pub struct InputActionsPlugin;

impl Plugin for InputActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputBindings>()
            .init_resource::<InputActionState>()
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

    for action in bindings.actions() {
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
