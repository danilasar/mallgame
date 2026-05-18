use bevy::prelude::*;

use crate::objects::components::{Deletable, Movable};
use crate::objects::rotation::Rotatable;
use crate::tools::{SelectionState, ToolChangedRequested, ToolMode};
use crate::ui::inspector::{
    auto_open_inspector_system, inspector_button_system, render_object_inspector,
};
use crate::ui::{
    ActiveInterfacePanel, BlocksWorldInput, InterfacePanelId, RightDockLayer, UiSet, UiWindowStack,
    WindowLayer,
    buttons::{UiFonts, label_text, ui_button},
};

#[derive(Component, Debug, Clone, Copy)]
pub struct InterfaceSwitcherButton {
    pub target: InterfacePanelId,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ToolModeUiButton {
    pub mode: ToolMode,
}

#[derive(Component, Debug, Clone, Copy)]
struct InterfacePanelContent;

pub struct RightDockUiPlugin;

impl Plugin for RightDockUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_right_dock.after(crate::ui::setup_ui_root))
            .add_systems(
                Update,
                (
                    right_dock_button_system,
                    render_active_interface_panel,
                    tool_mode_ui_button_system,
                    inspector_button_system,
                    auto_open_inspector_system,
                )
                    .chain()
                    .in_set(UiSet::Requests),
            );
    }
}

fn setup_right_dock(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    layer: Query<Entity, With<RightDockLayer>>,
) {
    let Some(layer) = layer.iter().next() else {
        return;
    };

    let dock = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(96.0),
                width: Val::Px(82.0),
                height: Val::Px(276.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            crate::ui::BlocksWorldInput,
            Interaction::default(),
            Name::new("RightDock"),
        ))
        .id();
    commands.entity(layer).add_child(dock);

    for (label, target) in [
        ("Tools", InterfacePanelId::Tools),
        ("Build", InterfacePanelId::BuildCatalog),
        ("Inventory", InterfacePanelId::Inventory),
        ("Debug", InterfacePanelId::Debug),
        ("Inspect", InterfacePanelId::ObjectInspector),
        ("Settings", InterfacePanelId::Settings),
    ] {
        let button = commands
            .spawn((
                ui_button(label, 82.0, 38.0),
                InterfaceSwitcherButton { target },
            ))
            .with_child(label_text(label, &fonts))
            .id();
        commands.entity(dock).add_child(button);
    }
}

fn right_dock_button_system(
    mut active: ResMut<ActiveInterfacePanel>,
    mut windows: ResMut<UiWindowStack>,
    mut query: Query<
        (&Interaction, &InterfaceSwitcherButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, button, mut color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                active.id = if active.id == Some(button.target) {
                    None
                } else {
                    Some(button.target)
                };
                windows.windows.clear();
                if let Some(panel) = active.id {
                    windows.windows.push(crate::ui::UiWindowInstance { panel });
                }
                if let Some(window) = windows.windows.last() {
                    info!("Active interface panel: {:?}", window.panel);
                }
                color.0 = Color::srgb(0.32, 0.25, 0.14);
            }
            Interaction::Hovered => color.0 = Color::srgb(0.24, 0.21, 0.18),
            Interaction::None => {
                color.0 = if active.id == Some(button.target) {
                    Color::srgb(0.28, 0.24, 0.16)
                } else {
                    Color::srgb(0.18, 0.16, 0.14)
                };
            }
        }
    }
}

fn render_active_interface_panel(
    mut commands: Commands,
    active: Res<ActiveInterfacePanel>,
    selection: Res<SelectionState>,
    tool_mode: Res<State<ToolMode>>,
    fonts: Res<UiFonts>,
    layer: Query<Entity, With<WindowLayer>>,
    existing: Query<Entity, With<InterfacePanelContent>>,
    objects: Query<(
        Option<&Name>,
        Option<&Movable>,
        Option<&Rotatable>,
        Option<&Deletable>,
    )>,
) {
    if !active.is_changed() && !selection.is_changed() && !tool_mode.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some(panel) = active.id else {
        return;
    };
    let Some(layer) = layer.iter().next() else {
        return;
    };

    let content = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(112.0),
                top: Val::Px(96.0),
                width: Val::Px(138.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.11, 0.10)),
            BlocksWorldInput,
            Interaction::default(),
            InterfacePanelContent,
            Name::new("InterfacePanelContent"),
        ))
        .id();
    commands.entity(layer).add_child(content);

    match panel {
        InterfacePanelId::Tools => {
            for (label, mode) in [
                ("Cursor", ToolMode::Cursor),
                ("Move", ToolMode::Move),
                ("Delete", ToolMode::Delete),
                ("Build", ToolMode::Build),
                ("Expand", ToolMode::Expansion),
            ] {
                let button = commands
                    .spawn((ui_button(label, 118.0, 34.0), ToolModeUiButton { mode }))
                    .with_child(label_text(label, &fonts))
                    .id();
                commands.entity(content).add_child(button);
            }
        }
        InterfacePanelId::BuildCatalog => {
            commands.entity(content).with_children(|parent| {
                parent.spawn(label_text("Build catalog", &fonts));
                parent.spawn(label_text("Prototype: Chair", &fonts));
            });
        }
        InterfacePanelId::Inventory => {
            commands
                .entity(content)
                .with_child(label_text("Inventory", &fonts));
        }
        InterfacePanelId::Debug => {
            commands
                .entity(content)
                .with_child(label_text("Debug", &fonts));
        }
        InterfacePanelId::ObjectInspector => {
            render_object_inspector(
                &mut commands,
                content,
                &selection,
                &fonts,
                *tool_mode.get(),
                objects,
            );
        }
        InterfacePanelId::Settings => {
            commands
                .entity(content)
                .with_child(label_text("Settings", &fonts));
        }
    }
}

fn tool_mode_ui_button_system(
    mut query: Query<(&Interaction, &ToolModeUiButton), Changed<Interaction>>,
    mut next: ResMut<NextState<ToolMode>>,
    mut changed: MessageWriter<ToolChangedRequested>,
) {
    for (interaction, button) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        changed.write(ToolChangedRequested { mode: button.mode });
        next.set(button.mode);
    }
}
