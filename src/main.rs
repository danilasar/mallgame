mod input;
mod objects;
mod placement;
mod presentation;
mod tools;
mod ui;

use bevy::prelude::*;
use input::*;
use objects::components::*;
use objects::prototypes::*;
use presentation::*;
use tools::*;
use ui::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.14)))
        .insert_resource(IsoProjection::default())
        .init_resource::<PointerContext>()
        .init_resource::<PointerDragState>()
        .init_resource::<ModalState>()
        .init_resource::<BuildPrototypes>()
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
            ToolCorePlugin,
            CursorToolPlugin,
            MoveToolPlugin,
            DeleteToolPlugin,
            BuildToolPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (
                update_pointer_context,
                update_hovered_object.after(update_pointer_context),
            ),
        )
        .add_systems(
            Update,
            (
                modal_input_system,
                camera_drag_system.after(modal_input_system),
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                update_highlight_intents,
                sync_visual_transform.before(TransformSystems::Propagate),
                update_highlight_visuals.after(sync_visual_transform),
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let floor = asset_server.load("floor.png");
    commands.spawn((
        Sprite {
            image: floor,
            custom_size: Some(Vec2::new(1600.0, 920.0)),
            ..default()
        },
        WorldPos(Vec2::ZERO),
        ProjectedPos(Vec2::ZERO),
        FootAnchor(Vec2::ZERO),
        VisualOffset(Vec2::ZERO),
        SortLayer::Floor,
        SortBias(0.0),
        PlaceableAssetId("floor/background"),
    ));

    spawn_object_from_prototype(
        &mut commands,
        &asset_server,
        BuildPrototypeId::Chair,
        Vec2::new(-180.0, -20.0),
    );
    spawn_object_from_prototype(
        &mut commands,
        &asset_server,
        BuildPrototypeId::Table,
        Vec2::new(80.0, -40.0),
    );
    spawn_object_from_prototype(
        &mut commands,
        &asset_server,
        BuildPrototypeId::Tree,
        Vec2::new(140.0, 130.0),
    );
}
