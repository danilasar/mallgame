mod input;
mod objects;
mod placement;
mod presentation;
mod save;
mod store;
mod tools;
mod ui;

use bevy::prelude::*;
use input::*;
use objects::components::*;
use objects::prototypes::*;
use objects::rotation::*;
use presentation::*;
use save::*;
use store::*;
use store::commands::*;
use tools::*;
use ui::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.14)))
        .insert_resource(IsoProjection::default())
        .init_resource::<PointerContext>()
        .init_resource::<PointerTargets>()
        .init_resource::<PointerDragState>()
        .init_resource::<BuildPrototypes>()
        .init_resource::<DomainCommandQueue>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Continuous 2D Isometric Tools Prototype".to_owned(),
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(InputActionsPlugin)
        .init_state::<ToolMode>()
        .add_plugins((
            UiCorePlugin,
            RightDockUiPlugin,
            BottomBuildPanelPlugin,
            CameraControlsUiPlugin,
            ModalUiPlugin,
            WorldWidgetUiPlugin,
            StorePlugin,
            StoreOverlayPlugin,
            SaveLoadPlugin,
        ))
        .add_plugins((
            ToolCorePlugin,
            CursorToolPlugin,
            MoveToolPlugin,
            DeleteToolPlugin,
            BuildToolPlugin,
            ExpansionToolPlugin,
            ObjectRotationPlugin,
            FootprintOverlayPlugin,
        ))
        .configure_sets(
            Update,
            (
                UiSet::UpdateInteraction,
                UiSet::Requests,
                UiSet::Modal,
                UiSet::WorldWidgets,
                ToolSet::InputGate,
                ToolSet::ToolUpdate,
                DomainCommandSet::RequestToCommand,
                DomainCommandSet::ApplyCommands,
                DomainCommandSet::EmitEvents,
                DomainCommandSet::PostDomainApply,
                ToolSet::Validation,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (
                update_pointer_context,
                update_pointer_over_ui,
                update_hovered_object.after(update_pointer_over_ui),
                camera_drag_system.after(update_hovered_object),
                update_tool_input_gate.after(camera_drag_system),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                handle_object_action_requests,
                handle_activate_tool_requested,
                handle_return_to_previous_tool_requested,
            )
                .chain()
                .in_set(ToolSet::ToolUpdate),
        )
        .add_systems(
            Update,
            (
                convert_committed_requests_to_commands,
                crate::objects::rotation::handle_rotate_requests,
                crate::store::expansion::convert_purchase_requests_to_commands,
            )
                .in_set(DomainCommandSet::RequestToCommand),
        )
        .add_systems(
            Update,
            apply_domain_commands.in_set(DomainCommandSet::ApplyCommands),
        )
        .add_systems(
            Update,
            unified_tool_validation_system.in_set(ToolSet::Validation),
        )
        .add_systems(
            Update,
            (
                log_tool_changed_requests,
                print_positions_system,
                handle_domain_event_selection_cleanup,
            )
                .in_set(DomainCommandSet::PostDomainApply),
        )
        .add_systems(
            PostUpdate,
            (
                update_highlight_intents,
                sync_visual_transform.before(TransformSystems::Propagate),
                update_highlight_visuals.after(sync_visual_transform),
                update_contextual_world_widgets.after(sync_visual_transform),
            )
                .chain(),
        );
    
    app.add_message::<crate::store::events::DomainEvent>();
    app.add_message::<crate::store::commands::DomainCommandRejected>();

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut allocator: ResMut<StableObjectIdAllocator>,
) {
    commands.spawn(Camera2d);
    let _sort_layers = SortLayer::ALL;

    spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        SpawnStoreObjectParams {
            stable_id: allocator.allocate(),
            prototype_id: BuildPrototypeId::Chair,
            world_pos: Vec2::new(-180.0, -20.0),
            rotation_index: Some(0),
        },
    );
    spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        SpawnStoreObjectParams {
            stable_id: allocator.allocate(),
            prototype_id: BuildPrototypeId::Table,
            world_pos: Vec2::new(80.0, -40.0),
            rotation_index: Some(0),
        },
    );
    spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        SpawnStoreObjectParams {
            stable_id: allocator.allocate(),
            prototype_id: BuildPrototypeId::Tree,
            world_pos: Vec2::new(140.0, 130.0),
            rotation_index: Some(0),
        },
    );
}
