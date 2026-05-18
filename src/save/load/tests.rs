use super::*;
use crate::objects::components::*;
use crate::objects::prototypes::BuildObjectId;

#[test]
fn test_save_load_restores_capabilities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    crate::store::commands::register_test_messages(&mut app);

    // Setup catalog
    let commands = app.world_mut().commands();
    crate::objects::prototypes::setup_object_catalog(commands);
    app.update();

    let _catalog = app.world().resource::<ObjectCatalog>().clone();

    let save = SaveGame {
        version: CURRENT_SAVE_VERSION,
        next_object_id: 2000,
        store: StoreSave {
            owned_chunks: vec![],
        },
        objects: vec![ObjectSave {
            id: StableObjectId(1001),
            prototype_id: BuildObjectId::new("fixture.shelf.basic"),
            placement: ObjectPlacementSave::Floor {
                world_pos: WorldPosSave { x: 0.0, y: 0.0 },
                rotation_index: None,
            },
        }],
    };

    let plan = build_load_plan(save, &SaveLoadLimits::default(), &WorldBounds::default()).unwrap();

    // Mock resources for apply_load_plan
    app.insert_resource(StoreArea::new(Vec2::ZERO));
    app.insert_resource(StableObjectIdAllocator { next: 1 });

    // Simpler way in App tests: run it as a system
    app.world_mut().insert_resource(plan);
    app.add_systems(
        Update,
        |mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut store: ResMut<StoreArea>,
         mut allocator: ResMut<StableObjectIdAllocator>,
         catalog: Res<ObjectCatalog>,
         query: Query<Entity, With<StoreObject>>,
         plan_res: Res<LoadPlan>| {
            apply_load_plan(
                &mut commands,
                &asset_server,
                &mut store,
                &mut allocator,
                &catalog,
                &query,
                &WorldBounds::default(),
                plan_res.clone(),
            );
        },
    );

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&ObjectStableId, &ProductContainer, &NpcInteractionPoints)>();

    let mut found = false;
    for (sid, _, _) in query.iter(world) {
        if sid.0 == StableObjectId(1001) {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "Loaded object should have sid 1001 and all capability components"
    );
}

#[test]
fn test_save_load_restores_wall_mounted_object() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    crate::store::commands::register_test_messages(&mut app);

    let commands = app.world_mut().commands();
    crate::objects::prototypes::setup_object_catalog(commands);
    app.update();

    let segment_key = crate::store::WallSegmentKey {
        chunk: crate::store::StoreChunkCoord { x: -1, y: -1 },
        side: crate::store::StoreBoundarySide::Top,
    };
    let save = SaveGame {
        version: CURRENT_SAVE_VERSION,
        next_object_id: 2000,
        store: StoreSave {
            owned_chunks: vec![],
        },
        objects: vec![ObjectSave {
            id: StableObjectId(3001),
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: ObjectPlacementSave::WallMounted {
                segment_key: WallSegmentKeySave {
                    chunk: segment_key.chunk,
                    side: segment_key.side,
                },
                offset_along_segment: 64.0,
                height_on_wall: 48.0,
            },
        }],
    };

    let plan = build_load_plan(save, &SaveLoadLimits::default(), &WorldBounds::default()).unwrap();

    app.insert_resource(StoreArea::new(Vec2::ZERO));
    app.insert_resource(StableObjectIdAllocator { next: 1 });
    app.world_mut().insert_resource(plan);
    app.add_systems(
        Update,
        |mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut store: ResMut<StoreArea>,
         mut allocator: ResMut<StableObjectIdAllocator>,
         catalog: Res<ObjectCatalog>,
         query: Query<Entity, With<StoreObject>>,
         plan_res: Res<LoadPlan>| {
            apply_load_plan(
                &mut commands,
                &asset_server,
                &mut store,
                &mut allocator,
                &catalog,
                &query,
                &WorldBounds::default(),
                plan_res.clone(),
            );
        },
    );

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &ObjectStableId,
        &WallMounted,
        &crate::objects::components::WallMountedPlacement,
        &crate::objects::components::Wallprint,
        &WallMountedBounds,
        &ObjectPlacementComponent,
        Option<&Footprint>,
        Option<&crate::objects::components::BlocksPlacement>,
        Option<&crate::objects::components::Movable>,
        Option<&crate::objects::components::WallMovable>,
        &crate::objects::components::Selectable,
        &crate::objects::components::Inspectable,
        &crate::objects::components::Deletable,
        &StoreObject,
    )>();
    let found = query.iter(world).any(
        |(
            sid,
            mounted,
            mounted_placement,
            wallprint,
            bounds,
            placement,
            footprint,
            blocks_placement,
            movable,
            wall_movable,
            _,
            _,
            _,
            _store_object,
        )| {
            sid.0 == StableObjectId(3001)
                && mounted.attachment.segment_key == segment_key
                && mounted_placement.attachment.segment_key == segment_key
                && wallprint.rects.len() == 1
                && wallprint.rects[0].segment_key == segment_key
                && bounds.segment_key == segment_key
                && footprint.is_none()
                && blocks_placement.is_none()
                && movable.is_none()
                && wall_movable.is_some()
                && matches!(
                    placement.placement,
                    ObjectPlacement::WallMounted { attachment }
                        if attachment.segment_key == segment_key
                )
        },
    );

    assert!(
        found,
        "wall-mounted object should restore from save placement"
    );
}
