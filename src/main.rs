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
use objects::rotation::ObjectRotationPlugin;
use presentation::*;
use save::*;
use store::*;
use tools::*;
use ui::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.14)))
        .insert_resource(IsoProjection::default())
        .init_resource::<PointerContext>()
        .init_resource::<PointerTargets>()
        .init_resource::<PointerDragState>()
        .init_resource::<DomainCommandQueue>()
        .init_resource::<HighlightRuntimeState>()
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
            BuildRibbonPlugin,
            CameraControlsUiPlugin,
            ModalUiPlugin,
            WorldWidgetUiPlugin,
            StorePlugin,
            StoreBoundaryPlugin,
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
        .add_systems(Startup, (setup_object_catalog, setup).chain())
        .add_systems(
            PreUpdate,
            (
                update_pointer_context,
                update_pointer_over_ui,
                update_hovered_object,
                camera_drag_system,
                update_tool_input_gate,
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
                ApplyDeferred,
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
    catalog: Res<ObjectCatalog>,
    mut allocator: ResMut<StableObjectIdAllocator>,
) {
    commands.spawn(Camera2d);
    let _sort_layers = SortLayer::ALL;

    let prototypes = [
        ("fixture.shelf.basic", Vec2::new(-180.0, -20.0)),
        ("service.checkout.basic", Vec2::new(80.0, -40.0)),
        ("decor.plant.tree", Vec2::new(140.0, 130.0)),
        ("wall.decor.placeholder", Vec2::new(200.0, 120.0)),
    ];

    for (proto_id, pos) in prototypes {
        if let Err(e) = spawn_store_object_from_prototype(
            &mut commands,
            &asset_server,
            &catalog,
            SpawnStoreObjectParams {
                stable_id: allocator.allocate(),
                prototype_id: BuildObjectId::new(proto_id),
                world_pos: pos,
                rotation_index: Some(0),
            },
        ) {
            warn!("Failed to spawn startup object: {}", e);
        }
    }
}
