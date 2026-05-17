use bevy::prelude::*;

use crate::objects::prototypes::BuildPrototypeId;
use crate::tools::{SelectBuildObjectRequested, ToolChangedRequested, ToolMode};
use crate::ui::{
    BlocksWorldInput, UiRoot, UiSet,
    buttons::{UiFonts, label_text, ui_button, ui_text},
};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPanelMode {
    Objects,
    Expansion,
    Closed,
}

impl Default for BuildPanelMode {
    fn default() -> Self {
        Self::Objects
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct BottomBuildPanel;

#[derive(Component, Debug, Clone, Copy)]
struct BuildPanelModeButton {
    mode: BuildPanelMode,
}

#[derive(Component, Debug, Clone, Copy)]
struct BuildPanelCloseButton;

#[derive(Component, Debug, Clone, Copy)]
struct BuildObjectInstallButton {
    prototype: BuildPrototypeId,
}

pub struct BottomBuildPanelPlugin;

impl Plugin for BottomBuildPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildPanelMode>()
            .add_systems(
                Startup,
                setup_bottom_build_panel.after(crate::ui::setup_ui_root),
            )
            .add_systems(
                Update,
                (
                    build_panel_mode_buttons,
                    build_panel_close_button,
                    build_object_install_buttons,
                    render_bottom_build_panel,
                )
                    .chain()
                    .in_set(UiSet::Requests),
            );
    }
}

fn setup_bottom_build_panel(mut commands: Commands, root: Query<Entity, With<UiRoot>>) {
    let Some(root) = root.iter().next() else {
        return;
    };

    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                right: Val::Px(118.0),
                bottom: Val::Px(18.0),
                height: Val::Px(134.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.105, 0.09, 0.94)),
            BlocksWorldInput,
            Interaction::default(),
            BottomBuildPanel,
            Name::new("BottomBuildPanel"),
        ))
        .id();
    commands.entity(root).add_child(panel);
}

fn build_panel_mode_buttons(
    mut mode: ResMut<BuildPanelMode>,
    mut query: Query<(&Interaction, &BuildPanelModeButton), Changed<Interaction>>,
    mut next_tool: ResMut<NextState<ToolMode>>,
    mut changed: MessageWriter<ToolChangedRequested>,
) {
    for (interaction, button) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *mode = button.mode;
        match button.mode {
            BuildPanelMode::Objects => {
                next_tool.set(ToolMode::Cursor);
                changed.write(ToolChangedRequested {
                    mode: ToolMode::Cursor,
                });
            }
            BuildPanelMode::Expansion => {
                next_tool.set(ToolMode::Expansion);
                changed.write(ToolChangedRequested {
                    mode: ToolMode::Expansion,
                });
            }
            BuildPanelMode::Closed => {}
        }
    }
}

fn build_panel_close_button(
    mut mode: ResMut<BuildPanelMode>,
    mut query: Query<&Interaction, (With<BuildPanelCloseButton>, Changed<Interaction>)>,
    mut next_tool: ResMut<NextState<ToolMode>>,
    mut changed: MessageWriter<ToolChangedRequested>,
) {
    for interaction in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *mode = BuildPanelMode::Closed;
        next_tool.set(ToolMode::Cursor);
        changed.write(ToolChangedRequested {
            mode: ToolMode::Cursor,
        });
    }
}

fn build_object_install_buttons(
    mut query: Query<(&Interaction, &BuildObjectInstallButton), Changed<Interaction>>,
    mut requests: MessageWriter<SelectBuildObjectRequested>,
) {
    for (interaction, button) in &mut query {
        if *interaction == Interaction::Pressed {
            requests.write(SelectBuildObjectRequested {
                prototype: button.prototype,
            });
        }
    }
}

fn render_bottom_build_panel(
    mut commands: Commands,
    mode: Res<BuildPanelMode>,
    mut rendered_mode: Local<Option<BuildPanelMode>>,
    fonts: Res<UiFonts>,
    panels: Query<Entity, With<BottomBuildPanel>>,
    children: Query<&Children>,
) {
    if rendered_mode.is_some_and(|rendered| rendered == *mode) {
        return;
    }
    *rendered_mode = Some(*mode);
    let Some(panel) = panels.iter().next() else {
        return;
    };

    if let Ok(children) = children.get(panel) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    match *mode {
        BuildPanelMode::Closed => {
            commands.entity(panel).insert(Node {
                display: Display::None,
                ..default()
            });
        }
        BuildPanelMode::Objects => {
            commands.entity(panel).insert(Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                right: Val::Px(118.0),
                bottom: Val::Px(18.0),
                height: Val::Px(134.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            });
            render_objects_panel(&mut commands, panel, &fonts);
        }
        BuildPanelMode::Expansion => {
            commands.entity(panel).insert(Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                right: Val::Px(118.0),
                bottom: Val::Px(18.0),
                height: Val::Px(82.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            });
            render_expansion_panel(&mut commands, panel, &fonts);
        }
    }
}

fn render_objects_panel(commands: &mut Commands, panel: Entity, fonts: &UiFonts) {
    let header = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(panel).add_child(header);
    commands.entity(header).with_children(|row| {
        row.spawn((
            ui_button("Objects", 86.0, 30.0),
            BuildPanelModeButton {
                mode: BuildPanelMode::Objects,
            },
        ))
        .with_child(label_text("Objects", fonts));
        row.spawn((
            ui_button("Expansion", 104.0, 30.0),
            BuildPanelModeButton {
                mode: BuildPanelMode::Expansion,
            },
        ))
        .with_child(label_text("Expansion", fonts));
        row.spawn((ui_button("X", 34.0, 30.0), BuildPanelCloseButton))
            .with_child(label_text("X", fonts));
    });

    let catalog = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .id();
    commands.entity(panel).add_child(catalog);

    for (name, prototype) in [
        ("Chair", BuildPrototypeId::Chair),
        ("Table", BuildPrototypeId::Table),
        ("Tree", BuildPrototypeId::Tree),
    ] {
        let card = commands
            .spawn((
                Node {
                    width: Val::Px(142.0),
                    height: Val::Px(68.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.18, 0.16, 0.14)),
                BlocksWorldInput,
                Interaction::default(),
            ))
            .id();
        commands.entity(catalog).add_child(card);
        commands.entity(card).with_children(|parent| {
            parent.spawn(label_text(name, fonts));
            parent
                .spawn((
                    ui_button("Install", 104.0, 28.0),
                    BuildObjectInstallButton { prototype },
                ))
                .with_child(label_text("Install", fonts));
        });
    }
}

fn render_expansion_panel(commands: &mut Commands, panel: Entity, fonts: &UiFonts) {
    let header = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(panel).add_child(header);
    commands.entity(header).with_children(|row| {
        row.spawn((
            ui_button("Objects", 86.0, 30.0),
            BuildPanelModeButton {
                mode: BuildPanelMode::Objects,
            },
        ))
        .with_child(label_text("Objects", fonts));
        row.spawn((
            ui_button("Expansion", 104.0, 30.0),
            BuildPanelModeButton {
                mode: BuildPanelMode::Expansion,
            },
        ))
        .with_child(label_text("Expansion", fonts));
        row.spawn((ui_button("X", 34.0, 30.0), BuildPanelCloseButton))
            .with_child(label_text("X", fonts));
    });
    commands.entity(panel).with_children(|parent| {
        parent.spawn(ui_text(
            "Выберите доступный блок 4x4 на карте",
            14.0,
            Color::srgb(0.94, 0.86, 0.58),
            fonts,
        ));
    });
}
