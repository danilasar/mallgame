use bevy::prelude::*;

use crate::input::PointerContext;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSet {
    UpdateInteraction,
    Requests,
    Modal,
    WorldWidgets,
}

#[derive(Resource, Debug, Default)]
pub struct UiRuntime {
    pub pointer_over_ui: bool,
}

#[derive(Resource, Debug, Default)]
pub struct UiWindowStack {
    pub windows: Vec<UiWindowInstance>,
}

#[derive(Debug, Clone)]
pub struct UiWindowInstance {
    pub panel: InterfacePanelId,
}

#[derive(Resource, Debug, Default)]
pub struct ActiveInterfacePanel {
    pub id: Option<InterfacePanelId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterfacePanelId {
    Tools,
    BuildCatalog,
    Inventory,
    ObjectInspector,
    Debug,
    Settings,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UiRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct WorldWidgetsLayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct RightDockLayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct WindowLayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct ModalLayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct BlocksWorldInput;

pub struct UiCorePlugin;

impl Plugin for UiCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiRuntime>()
            .init_resource::<UiWindowStack>()
            .init_resource::<ActiveInterfacePanel>()
            .configure_sets(
                Update,
                (
                    UiSet::UpdateInteraction,
                    UiSet::Requests,
                    UiSet::Modal,
                    UiSet::WorldWidgets,
                )
                    .chain(),
            )
            .add_systems(Startup, setup_ui_root)
            .add_systems(
                Update,
                update_pointer_over_ui.in_set(UiSet::UpdateInteraction),
            );
    }
}

pub fn setup_ui_root(mut commands: Commands) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            UiRoot,
            Name::new("UiRoot"),
        ))
        .id();

    let world_widgets = commands
        .spawn((
            empty_layer_node(),
            WorldWidgetsLayer,
            Name::new("WorldWidgetsLayer"),
        ))
        .id();
    let right_dock = commands
        .spawn((
            empty_layer_node(),
            RightDockLayer,
            Name::new("RightDockLayer"),
        ))
        .id();
    let window_layer = commands
        .spawn((empty_layer_node(), WindowLayer, Name::new("WindowLayer")))
        .id();
    let modal_layer = commands
        .spawn((empty_layer_node(), ModalLayer, Name::new("ModalLayer")))
        .id();

    commands
        .entity(root)
        .add_children(&[world_widgets, right_dock, window_layer, modal_layer]);
}

fn empty_layer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

pub fn update_pointer_over_ui(
    mut runtime: ResMut<UiRuntime>,
    mut pointer: ResMut<PointerContext>,
    query: Query<&Interaction, With<BlocksWorldInput>>,
) {
    let pointer_over_ui = query
        .iter()
        .any(|interaction| matches!(*interaction, Interaction::Hovered | Interaction::Pressed));

    runtime.pointer_over_ui = pointer_over_ui;
    pointer.over_ui = pointer_over_ui;
}
