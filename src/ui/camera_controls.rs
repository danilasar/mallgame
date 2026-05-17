use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::input::{InputAction, InputActionState};
use crate::ui::ModalStack;
use crate::ui::{
    RightDockLayer, UiRuntime, UiSet,
    buttons::{label_text, ui_button},
};

#[derive(Component, Debug, Clone, Copy)]
pub enum CameraControlButton {
    ZoomIn,
    ZoomOut,
    ToggleFullscreen,
}

#[derive(Message, Debug, Clone, Copy)]
pub enum CameraControlRequested {
    ZoomIn,
    ZoomOut,
    ToggleFullscreen,
}

pub struct CameraControlsUiPlugin;

impl Plugin for CameraControlsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CameraControlRequested>()
            .add_systems(
                Startup,
                setup_camera_controls.after(crate::ui::setup_ui_root),
            )
            .add_systems(
                Update,
                (
                    camera_control_button_system,
                    camera_control_keyboard_system,
                    camera_control_wheel_system,
                    apply_camera_control_requests,
                )
                    .chain()
                    .in_set(UiSet::Requests),
            );
    }
}

fn setup_camera_controls(mut commands: Commands, layer: Query<Entity, With<RightDockLayer>>) {
    let Some(layer) = layer.iter().next() else {
        return;
    };

    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(130.0),
                height: Val::Px(34.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            },
            crate::ui::BlocksWorldInput,
            Interaction::default(),
            Name::new("CameraControls"),
        ))
        .id();
    commands.entity(layer).add_child(panel);

    for (label, control) in [
        ("+", CameraControlButton::ZoomIn),
        ("-", CameraControlButton::ZoomOut),
        ("[]", CameraControlButton::ToggleFullscreen),
    ] {
        let button = commands
            .spawn((ui_button(label, 38.0, 34.0), control))
            .with_child(label_text(label))
            .id();
        commands.entity(panel).add_child(button);
    }
}

fn camera_control_button_system(
    mut query: Query<(&Interaction, &CameraControlButton), Changed<Interaction>>,
    mut requests: MessageWriter<CameraControlRequested>,
) {
    for (interaction, button) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        requests.write(match button {
            CameraControlButton::ZoomIn => CameraControlRequested::ZoomIn,
            CameraControlButton::ZoomOut => CameraControlRequested::ZoomOut,
            CameraControlButton::ToggleFullscreen => CameraControlRequested::ToggleFullscreen,
        });
    }
}

fn camera_control_keyboard_system(
    actions: Res<InputActionState>,
    mut requests: MessageWriter<CameraControlRequested>,
) {
    if actions.just_pressed(InputAction::CameraZoomIn) {
        requests.write(CameraControlRequested::ZoomIn);
    }
    if actions.just_pressed(InputAction::CameraZoomOut) {
        requests.write(CameraControlRequested::ZoomOut);
    }
    if actions.just_pressed(InputAction::ToggleFullscreen) {
        requests.write(CameraControlRequested::ToggleFullscreen);
    }
}

fn camera_control_wheel_system(
    mut wheel: MessageReader<MouseWheel>,
    runtime: Res<UiRuntime>,
    modal: Res<ModalStack>,
    mut requests: MessageWriter<CameraControlRequested>,
) {
    if runtime.pointer_over_ui || !modal.stack.is_empty() {
        wheel.clear();
        return;
    }

    let mut amount = 0.0;
    for event in wheel.read() {
        amount += event.y;
    }

    if amount > 0.0 {
        requests.write(CameraControlRequested::ZoomIn);
    } else if amount < 0.0 {
        requests.write(CameraControlRequested::ZoomOut);
    }
}

fn apply_camera_control_requests(
    mut requests: MessageReader<CameraControlRequested>,
    mut projections: Query<&mut Projection, With<Camera2d>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for request in requests.read() {
        match request {
            CameraControlRequested::ZoomIn => {
                if let Some(mut projection) = projections.iter_mut().next() {
                    if let Projection::Orthographic(orthographic) = &mut *projection {
                        orthographic.scale = (orthographic.scale * 0.9).clamp(0.25, 4.0);
                    }
                }
            }
            CameraControlRequested::ZoomOut => {
                if let Some(mut projection) = projections.iter_mut().next() {
                    if let Projection::Orthographic(orthographic) = &mut *projection {
                        orthographic.scale = (orthographic.scale * 1.1).clamp(0.25, 4.0);
                    }
                }
            }
            CameraControlRequested::ToggleFullscreen => {
                if let Some(mut window) = windows.iter_mut().next() {
                    window.mode = match window.mode {
                        WindowMode::Windowed => {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        }
                        _ => WindowMode::Windowed,
                    };
                }
            }
        }
    }
}
