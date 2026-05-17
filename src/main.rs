mod components;
mod input;
mod placement;
mod presentation;
mod projection;

use bevy::prelude::*;
use components::*;
use input::*;
use placement::*;
use presentation::*;
use projection::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.14)))
        .insert_resource(IsoProjection::default())
        .insert_resource(DragState::default())
        .insert_resource(PlacementAreas::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Continuous 2D Isometric Prototype".to_owned(),
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                movement_system,
                select_and_begin_drag_system,
                drag_system,
                end_drag_system,
                apply_selection_tint_system,
                print_positions_system,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            sync_visual_transform_system.before(TransformSystems::Propagate),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let floor = asset_server.load("floor.png");
    let chair = asset_server.load("chair.png");
    let table = asset_server.load("table.png");
    let tree = asset_server.load("tree.png");

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

    spawn_placeable(
        &mut commands,
        chair,
        "chair",
        Vec2::new(-180.0, -20.0),
        Vec2::new(96.0, 128.0),
        Vec2::new(0.0, -48.0),
        Vec2::new(26.0, 18.0),
        -0.2,
    );
    spawn_placeable(
        &mut commands,
        table,
        "table",
        Vec2::new(80.0, -40.0),
        Vec2::new(160.0, 128.0),
        Vec2::new(0.0, -42.0),
        Vec2::new(54.0, 32.0),
        0.0,
    );
    spawn_placeable(
        &mut commands,
        tree,
        "tree",
        Vec2::new(140.0, 130.0),
        Vec2::new(144.0, 220.0),
        Vec2::new(0.0, -86.0),
        Vec2::new(32.0, 28.0),
        0.2,
    );
}

fn spawn_placeable(
    commands: &mut Commands,
    image: Handle<Image>,
    asset_id: &'static str,
    world_pos: Vec2,
    sprite_size: Vec2,
    foot_anchor: Vec2,
    footprint_half_extents: Vec2,
    sort_bias: f32,
) {
    commands
        .spawn((
            Sprite {
                image,
                custom_size: Some(sprite_size),
                ..default()
            },
            WorldPos(world_pos),
            Velocity::default(),
            ProjectedPos::default(),
            FootAnchor(foot_anchor),
            VisualOffset(Vec2::ZERO),
            SortLayer::Objects,
            SortBias(sort_bias),
            PlacementState::Placed,
            InteractionState::default(),
            Draggable,
            Selectable,
            SelectionTint {
                normal: Color::WHITE,
                selected: Color::srgb(1.0, 0.92, 0.55),
                dragging: Color::srgba(0.65, 0.90, 1.0, 0.82),
                blocked: Color::srgba(1.0, 0.35, 0.30, 0.88),
            },
        ))
        .insert((
            CollisionFootprint {
                half_extents: footprint_half_extents,
            },
            BlocksPlacement,
            PlaceableAssetId(asset_id),
        ));
}
