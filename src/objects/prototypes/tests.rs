use super::*;

#[test]
fn test_catalog_validation_missing_browse_point() {
    let mut catalog = ObjectCatalog::default();
    catalog.prototypes.insert(
        BuildObjectId::new("fail"),
        ObjectPrototype {
            id: BuildObjectId::new("fail"),
            display: ObjectDisplaySpec {
                display_name: "Fail".to_string(),
                description: None,
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Fixture,
                ribbon_tab: BuildRibbonTab::Fixtures,
                ribbon_group: BuildRibbonGroup::Shelves,
                sort_order: 0,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::Floor,
                footprint_half_extents: Vec2::ZERO,
                placement_blocker: false,
                navigation_blocker: false,
            },
            visuals: VisualSpec {
                asset_path: "".into(),
                asset_id: "".into(),
                sprite_size: Vec2::ZERO,
                foot_anchor: Vec2::ZERO,
                sort_bias: 0.0,
            },
            rotation: RotationSpec {
                kind: RotationKind::None,
                rotated_asset_path: None,
            },
            capabilities: vec![ObjectCapabilitySpec::ProductContainer(
                ProductContainerSpec {
                    container_kind: ProductContainerKind::Shelf,
                    capacity_class: ContainerCapacityClass::Small,
                },
            )],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    let errors = validate_object_catalog(&catalog);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("no BrowseProducts interaction point"))
    );
}

#[test]
fn test_factory_mapping_capabilities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    crate::store::commands::register_test_messages(&mut app);

    // Re-setup standard catalog for testing
    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>().clone();
    let asset_server = app.world().resource::<AssetServer>().clone();

    let mut commands = app.world_mut().commands();
    let proto_id = BuildObjectId::new("fixture.shelf.basic");
    let entity = spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        &catalog,
        SpawnStoreObjectParams {
            stable_id: StableObjectId(1),
            prototype_id: proto_id.clone(),
            placement: ObjectPlacement::Floor {
                world_pos: Vec2::ZERO,
                rotation_index: None,
            },
            derived_door: None,
        },
    )
    .expect("Spawn failed");

    app.update();

    let world = app.world();
    assert!(world.entity(entity).contains::<ProductContainer>());
    assert!(world.entity(entity).contains::<NpcInteractionPoints>());
    assert!(!world.entity(entity).contains::<CheckoutPoint>());
    assert!(world.entity(entity).contains::<StoreObject>());
    assert!(world.entity(entity).contains::<FloorPlacement>());
    assert!(world.entity(entity).contains::<Footprint>());
    assert!(!world.entity(entity).contains::<Wallprint>());
    assert!(world.entity(entity).contains::<Movable>());
}

#[test]
fn test_wall_mounted_factory_does_not_add_floor_geometry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();

    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>().clone();
    let asset_server = app.world().resource::<AssetServer>().clone();
    let segment_key = crate::store::WallSegmentKey {
        chunk: crate::store::StoreChunkCoord { x: -1, y: -1 },
        side: crate::store::StoreBoundarySide::Top,
    };

    let mut commands = app.world_mut().commands();
    let entity = spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        &catalog,
        SpawnStoreObjectParams {
            stable_id: StableObjectId(2),
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: ObjectPlacement::WallMounted {
                attachment: WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 64.0,
                    height_on_wall: 48.0,
                },
            },
            derived_door: None,
        },
    )
    .expect("Spawn failed");

    app.update();

    let world = app.world();
    assert!(world.entity(entity).contains::<StoreObject>());
    assert!(world.entity(entity).contains::<WallMountedPlacement>());
    assert!(world.entity(entity).contains::<WallMounted>());
    assert!(world.entity(entity).contains::<Wallprint>());
    assert!(world.entity(entity).contains::<WallMountedBounds>());
    assert!(!world.entity(entity).contains::<Footprint>());
    assert!(!world.entity(entity).contains::<BlocksPlacement>());
    assert!(!world.entity(entity).contains::<Movable>());
}

#[test]
fn test_wall_mounted_prototype_is_visible_in_walls_tab() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>();
    let proto = catalog
        .prototypes
        .get(&BuildObjectId::new("wall.decor.placeholder"))
        .expect("wall decor prototype should exist");

    assert_eq!(proto.placement.kind, PlacementKind::WallMounted);
    assert_eq!(proto.catalog.ribbon_tab, BuildRibbonTab::Walls);
    assert_eq!(proto.catalog.availability, CatalogAvailability::Available);

    let window = catalog
        .prototypes
        .get(&BuildObjectId::new("wall.window.basic_visual"))
        .expect("visual window prototype should exist");

    assert_eq!(window.placement.kind, PlacementKind::WallMounted);
    assert_eq!(window.catalog.ribbon_tab, BuildRibbonTab::Walls);
    assert!(
        window
            .capabilities
            .iter()
            .any(|cap| matches!(cap, ObjectCapabilitySpec::Window(WindowSpec { .. })))
    );
}
