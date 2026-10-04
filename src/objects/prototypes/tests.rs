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

// ── Stage 5B.7 tests ─────────────────────────────────────────────────────────

fn make_wall_proto(id: &str, capabilities: Vec<ObjectCapabilitySpec>) -> ObjectPrototype {
    ObjectPrototype {
        id: BuildObjectId::new(id),
        display: ObjectDisplaySpec {
            display_name: id.to_string(),
            description: None,
            icon: None,
        },
        catalog: ObjectCatalogSpec {
            category: ObjectCategory::Decor,
            ribbon_tab: BuildRibbonTab::Walls,
            ribbon_group: BuildRibbonGroup::Walls,
            sort_order: 0,
            availability: CatalogAvailability::Available,
        },
        placement: PlacementSpec {
            kind: PlacementKind::WallMounted,
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
        capabilities,
        initial_state: ObjectInitialStateSpec::None,
    }
}

fn wall_mounted_spec_cap() -> ObjectCapabilitySpec {
    ObjectCapabilitySpec::WallMounted(WallMountedSpec {
        width: 72.0,
        height: 72.0,
        allowed_sides: vec![crate::store::StoreBoundarySide::Top],
        default_height_on_wall: 0.0,
        movable: true,
    })
}

fn test_attachment() -> crate::objects::components::WallAttachmentPoint {
    crate::objects::components::WallAttachmentPoint {
        segment_key: crate::store::WallSegmentKey {
            chunk: crate::store::StoreChunkCoord { x: 0, y: 0 },
            side: crate::store::StoreBoundarySide::Top,
        },
        offset_along_segment: 100.0,
        height_on_wall: 56.0,
    }
}

#[test]
fn test_wall_opening_requires_wall_mounted_visual() {
    let mut catalog = ObjectCatalog::default();
    catalog.prototypes.insert(
        BuildObjectId::new("t"),
        make_wall_proto(
            "t",
            vec![
                wall_mounted_spec_cap(),
                ObjectCapabilitySpec::WallOpening(WallOpeningSpec {
                    shape: WallOpeningShapeSpec::Rect {
                        width: 72.0,
                        height: 72.0,
                        anchor: WallOpeningAnchor::BottomCenter,
                    },
                    glass_color: None,
                    frame_color: None,
                }),
            ],
        ),
    );
    let errors = validate_object_catalog(&catalog);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("WallOpening") && e.contains("WallMountedVisual")),
        "expected error about WallOpening requiring WallMountedVisual, got: {:?}",
        errors
    );
}

#[test]
fn test_wall_mounted_visual_invalid_dimensions_rejected() {
    let mut catalog = ObjectCatalog::default();
    catalog.prototypes.insert(
        BuildObjectId::new("t"),
        make_wall_proto(
            "t",
            vec![
                wall_mounted_spec_cap(),
                ObjectCapabilitySpec::WallMountedVisual(WallMountedVisualSpec {
                    visual_width: 0.0,
                    visual_height: -1.0,
                    anchor: WallVisualAnchor::BottomCenter,
                    offset: Vec2::ZERO,
                }),
            ],
        ),
    );
    let errors = validate_object_catalog(&catalog);
    assert!(errors.iter().any(|e| e.contains("visual_width <= 0")));
    assert!(errors.iter().any(|e| e.contains("visual_height <= 0")));
}

#[test]
fn test_wall_mounted_visual_on_floor_object_rejected() {
    let mut catalog = ObjectCatalog::default();
    let mut proto = make_wall_proto(
        "t",
        vec![ObjectCapabilitySpec::WallMountedVisual(WallMountedVisualSpec {
            visual_width: 72.0,
            visual_height: 72.0,
            anchor: WallVisualAnchor::BottomCenter,
            offset: Vec2::ZERO,
        })],
    );
    proto.placement.kind = PlacementKind::Floor;
    catalog.prototypes.insert(BuildObjectId::new("t"), proto);
    let errors = validate_object_catalog(&catalog);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("WallMountedVisual") && e.contains("not wall-mounted"))
    );
}

#[test]
fn test_visual_bounds_warning_when_smaller_than_opening() {
    let mut catalog = ObjectCatalog::default();
    catalog.prototypes.insert(
        BuildObjectId::new("t"),
        make_wall_proto(
            "t",
            vec![
                wall_mounted_spec_cap(),
                ObjectCapabilitySpec::WallOpening(WallOpeningSpec {
                    shape: WallOpeningShapeSpec::Rect {
                        width: 72.0,
                        height: 72.0,
                        anchor: WallOpeningAnchor::BottomCenter,
                    },
                    glass_color: None,
                    frame_color: None,
                }),
                ObjectCapabilitySpec::WallMountedVisual(WallMountedVisualSpec {
                    visual_width: 50.0, // smaller than opening 72
                    visual_height: 50.0,
                    anchor: WallVisualAnchor::BottomCenter,
                    offset: Vec2::ZERO,
                }),
            ],
        ),
    );
    let errors = validate_object_catalog(&catalog);
    assert!(
        errors.iter().any(|e| e.starts_with("[Warning]") && e.contains("visual bounds")),
        "expected warning about visual bounds not covering opening, got: {:?}",
        errors
    );
}

#[test]
fn test_visual_bounds_no_warning_when_equal_to_opening() {
    let mut catalog = ObjectCatalog::default();
    catalog.prototypes.insert(
        BuildObjectId::new("t"),
        make_wall_proto(
            "t",
            vec![
                wall_mounted_spec_cap(),
                ObjectCapabilitySpec::WallOpening(WallOpeningSpec {
                    shape: WallOpeningShapeSpec::Rect {
                        width: 72.0,
                        height: 72.0,
                        anchor: WallOpeningAnchor::BottomCenter,
                    },
                    glass_color: None,
                    frame_color: None,
                }),
                ObjectCapabilitySpec::WallMountedVisual(WallMountedVisualSpec {
                    visual_width: 72.0,
                    visual_height: 72.0,
                    anchor: WallVisualAnchor::BottomCenter,
                    offset: Vec2::ZERO,
                }),
            ],
        ),
    );
    let errors = validate_object_catalog(&catalog);
    let hard: Vec<_> = errors
        .iter()
        .filter(|e| !e.starts_with("[Warning]"))
        .collect();
    assert!(hard.is_empty(), "no hard errors expected, got: {:?}", hard);
}

#[test]
fn test_standard_catalog_has_no_hard_errors() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>().clone();
    let errors = validate_object_catalog(&catalog);
    let hard: Vec<_> = errors
        .iter()
        .filter(|e| !e.starts_with("[Warning]"))
        .collect();
    assert!(
        hard.is_empty(),
        "standard catalog must have no hard validation errors, got: {:?}",
        hard
    );
}

#[test]
fn test_fake_window_has_no_wall_opening() {
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
        .get(&BuildObjectId::new("wall.window.fake_decor"))
        .expect("wall.window.fake_decor must exist");

    assert_eq!(proto.placement.kind, PlacementKind::WallMounted);
    assert!(
        !proto
            .capabilities
            .iter()
            .any(|c| matches!(c, ObjectCapabilitySpec::WallOpening(_))),
        "fake_decor must have no WallOpening"
    );
}

#[test]
fn test_poster_has_no_wall_opening() {
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
        .get(&BuildObjectId::new("wall.poster.basic"))
        .expect("wall.poster.basic must exist");

    assert_eq!(proto.placement.kind, PlacementKind::WallMounted);
    assert!(
        !proto
            .capabilities
            .iter()
            .any(|c| matches!(c, ObjectCapabilitySpec::WallOpening(_))),
        "poster must have no WallOpening"
    );
}

#[test]
fn test_derive_visual_bounds_bottom_center() {
    let vis = WallMountedVisualSpec {
        visual_width: 72.0,
        visual_height: 72.0,
        anchor: WallVisualAnchor::BottomCenter,
        offset: Vec2::ZERO,
    };
    let att = test_attachment();
    let vb = derive_visual_bounds(att, &vis);

    assert_eq!(vb.offset_min, att.offset_along_segment - 36.0);
    assert_eq!(vb.offset_max, att.offset_along_segment + 36.0);
    assert_eq!(vb.height_min, att.height_on_wall);
    assert_eq!(vb.height_max, att.height_on_wall + 72.0);
}

#[test]
fn test_derive_visual_bounds_center_anchor() {
    let vis = WallMountedVisualSpec {
        visual_width: 72.0,
        visual_height: 72.0,
        anchor: WallVisualAnchor::Center,
        offset: Vec2::ZERO,
    };
    let att = test_attachment(); // height_on_wall = 56
    let vb = derive_visual_bounds(att, &vis);

    assert_eq!(vb.height_min, att.height_on_wall - 36.0);
    assert_eq!(vb.height_max, att.height_on_wall + 36.0);
}

#[test]
fn test_derive_visual_bounds_with_offset() {
    let vis = WallMountedVisualSpec {
        visual_width: 72.0,
        visual_height: 72.0,
        anchor: WallVisualAnchor::BottomCenter,
        offset: Vec2::new(10.0, 5.0),
    };
    let att = test_attachment();
    let vb = derive_visual_bounds(att, &vis);

    assert_eq!(vb.offset_min, att.offset_along_segment - 36.0 + 10.0);
    assert_eq!(vb.height_min, att.height_on_wall + 5.0);
}

#[test]
fn test_visual_bounds_independent_from_opening_and_wallprint() {
    // Visual spec can specify larger dimensions than opening
    let vis = WallMountedVisualSpec {
        visual_width: 100.0,
        visual_height: 110.0,
        anchor: WallVisualAnchor::BottomCenter,
        offset: Vec2::ZERO,
    };
    let att = test_attachment();
    let vb = derive_visual_bounds(att, &vis);

    assert_eq!(vb.offset_max - vb.offset_min, 100.0);
    assert_eq!(vb.height_max - vb.height_min, 110.0);

    // These dimensions differ from a typical 72x72 opening
    assert_ne!(vb.offset_max - vb.offset_min, 72.0);
    assert_ne!(vb.height_max - vb.height_min, 72.0);
}

#[test]
fn test_factory_spawns_wall_visual_bounds_for_window() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    crate::store::commands::register_test_messages(&mut app);

    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>().clone();
    let asset_server = app.world().resource::<AssetServer>().clone();

    let mut commands = app.world_mut().commands();
    let entity = spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        &catalog,
        SpawnStoreObjectParams {
            stable_id: StableObjectId(10),
            prototype_id: BuildObjectId::new("wall.window.basic_visual"),
            placement: ObjectPlacement::WallMounted {
                attachment: test_attachment(),
            },
            derived_door: None,
        },
    )
    .expect("spawn failed");

    app.update();

    let world = app.world();
    assert!(
        world
            .entity(entity)
            .contains::<crate::objects::components::WallVisualBounds>(),
        "window must have WallVisualBounds"
    );
    assert!(
        world
            .entity(entity)
            .contains::<crate::objects::components::WallOpeningComponent>(),
        "window must have WallOpeningComponent"
    );
}

#[test]
fn test_factory_no_visual_bounds_for_poster() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    crate::store::commands::register_test_messages(&mut app);

    let commands = app.world_mut().commands();
    setup_object_catalog(commands);
    app.update();

    let catalog = app.world().resource::<ObjectCatalog>().clone();
    let asset_server = app.world().resource::<AssetServer>().clone();

    let mut commands = app.world_mut().commands();
    let entity = spawn_store_object_from_prototype(
        &mut commands,
        &asset_server,
        &catalog,
        SpawnStoreObjectParams {
            stable_id: StableObjectId(11),
            prototype_id: BuildObjectId::new("wall.poster.basic"),
            placement: ObjectPlacement::WallMounted {
                attachment: test_attachment(),
            },
            derived_door: None,
        },
    )
    .expect("spawn failed");

    app.update();

    let world = app.world();
    assert!(
        !world
            .entity(entity)
            .contains::<crate::objects::components::WallVisualBounds>(),
        "poster must NOT have WallVisualBounds (no WallMountedVisualSpec)"
    );
    assert!(
        !world
            .entity(entity)
            .contains::<crate::objects::components::WallOpeningComponent>(),
        "poster must NOT have WallOpeningComponent"
    );
}
