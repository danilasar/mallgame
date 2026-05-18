use super::*;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_resource::<DomainCommandQueue>();
    register_test_messages(&mut app);

    app.insert_resource(crate::store::WorldBounds {
        rect: Rect::from_corners(Vec2::new(-2000.0, -2000.0), Vec2::new(2000.0, 2000.0)),
    });

    let mut store = crate::store::area::StoreArea::new(Vec2::ZERO);
    // Manually add some chunks so placement is valid
    for x in -2..2 {
        for y in -2..2 {
            store.owned_chunks.insert(
                StoreChunkCoord { x, y },
                crate::store::chunks::StoreChunkData {
                    kind: StoreChunkKind::Default,
                },
            );
        }
    }
    app.insert_resource(store);

    app.insert_resource(crate::objects::components::StableObjectIdAllocator { next: 1000 });

    // Setup catalog
    let commands = app.world_mut().commands();
    crate::objects::prototypes::setup_object_catalog(commands);
    app.update(); // Run startup systems

    app
}

fn test_segment_key() -> crate::store::WallSegmentKey {
    crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    }
}

fn spawn_test_wall_surface(app: &mut App, segment_key: crate::store::WallSegmentKey) {
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(256.0, 0.0),
        length: 256.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });
}

fn object_entity_by_id(world: &mut World, object_id: StableObjectId) -> Option<Entity> {
    let mut query = world.query::<(Entity, &crate::objects::components::ObjectStableId)>();
    query
        .iter(world)
        .find_map(|(entity, id)| (id.0 == object_id).then_some(entity))
}

fn object_exists(world: &mut World, object_id: StableObjectId) -> bool {
    object_entity_by_id(world, object_id).is_some()
}

fn queue_build_door(
    app: &mut App,
    object_id: StableObjectId,
    segment_key: crate::store::WallSegmentKey,
    offset: f32,
) {
    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id,
            prototype_id: BuildObjectId::new("wall.door.basic_customer"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: offset,
                    height_on_wall: 96.0,
                },
            },
        }));
}

fn queue_build_shelf(app: &mut App, object_id: StableObjectId, world_pos: Vec2) {
    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id,
            prototype_id: BuildObjectId::new("fixture.shelf.basic"),
            placement: crate::objects::components::ObjectPlacement::Floor {
                world_pos,
                rotation_index: Some(0),
            },
        }));
}

#[test]
fn test_delete_object_command() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(102);

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());

    app.world_mut().spawn((
        crate::objects::components::ObjectStableId(object_id),
        crate::objects::components::ObjectPrototypeId(
            crate::objects::prototypes::BuildObjectId::new("test.dummy"),
        ),
        crate::objects::components::WorldPos(Vec2::ZERO),
        crate::objects::components::StoreObject,
    ));

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::DeleteObject(DeleteObjectCommand {
            object_id,
        }));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&crate::objects::components::ObjectStableId>();
    let found = query.iter(world).any(|id| id.0 == object_id);
    assert!(!found, "Object with ID 102 should have been deleted");
}

#[test]
fn test_rejection_on_collision() {
    let mut app = setup_test_app();
    let id1 = StableObjectId(201);
    let id2 = StableObjectId(202);
    let pos = Vec2::new(0.0, 0.0);

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());

    app.world_mut().spawn((
        crate::objects::components::ObjectStableId(id1),
        crate::objects::components::WorldPos(pos),
        crate::objects::components::Footprint::rectangle(Vec2::splat(20.0)),
        crate::objects::components::StoreObject,
        crate::objects::components::BlocksPlacement,
    ));

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id: id2,
            prototype_id: BuildObjectId::new("fixture.shelf.basic"),
            placement: crate::objects::components::ObjectPlacement::Floor {
                world_pos: pos,
                rotation_index: Some(0),
            },
        }));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&crate::objects::components::ObjectStableId>();
    let found = query.iter(world).any(|id| id.0 == id2);
    assert!(
        !found,
        "Object with ID 202 should NOT have been built due to collision"
    );
}

#[test]
fn test_build_pipeline_end_to_end() {
    let mut app = setup_test_app();
    let id = BuildObjectId::new("fixture.shelf.basic");
    let pos = Vec2::new(0.0, 0.0); // Inside manually added chunks

    app.add_systems(
        Update,
        (
            crate::tools::convert_committed_requests_to_commands,
            ApplyDeferred,
            apply_domain_commands,
            ApplyDeferred,
        )
            .chain(),
    );

    app.world_mut()
        .resource_scope(|_world, mut queue: Mut<DomainCommandQueue>| {
            queue
                .commands
                .push_back(DomainCommand::BuildObject(BuildObjectCommand {
                    object_id: StableObjectId(5000),
                    prototype_id: id.clone(),
                    placement: crate::objects::components::ObjectPlacement::Floor {
                        world_pos: pos,
                        rotation_index: Some(0),
                    },
                }));
        });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &crate::objects::components::ObjectPrototypeId,
        &crate::objects::components::ProductContainer,
        &crate::objects::components::NpcInteractionPoints,
    )>();

    let mut found = false;
    for (proto_id, _, _) in query.iter(world) {
        if proto_id.0 == id {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "Object should have been built with correct capabilities"
    );
}

#[test]
fn test_wall_mounted_build_spawns_store_object_without_floor_blocker() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(6000);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: -1, y: -1 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(-128.0, 0.0),
        end: Vec2::new(0.0, 0.0),
        length: 128.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id,
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 64.0,
                    height_on_wall: 48.0,
                },
            },
        }));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &crate::objects::components::ObjectStableId,
        &crate::objects::components::WallMounted,
        &crate::objects::components::WallMountedPlacement,
        &crate::objects::components::Wallprint,
        &crate::objects::components::WallMountedBounds,
        Option<&crate::objects::components::Footprint>,
        Option<&crate::objects::components::BlocksPlacement>,
        Option<&crate::objects::components::Movable>,
        &crate::objects::components::StoreObject,
    )>();
    let found = query.iter(world).any(
        |(
            stable_id,
            mounted,
            mounted_placement,
            wallprint,
            bounds,
            footprint,
            blocker,
            movable,
            _store_object,
        )| {
            stable_id.0 == object_id
                && mounted.attachment.segment_key == segment_key
                && mounted_placement.attachment.segment_key == segment_key
                && wallprint.rects.len() == 1
                && wallprint.rects[0].segment_key == segment_key
                && bounds.segment_key == segment_key
                && footprint.is_none()
                && blocker.is_none()
                && movable.is_none()
        },
    );

    assert!(found, "wall-mounted StoreObject was not spawned correctly");
}

#[test]
fn test_door_build_is_clamped_to_wall_floor() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(6003);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(128.0, 0.0),
        length: 128.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id,
            prototype_id: BuildObjectId::new("wall.door.basic_customer"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 64.0,
                    height_on_wall: 80.0,
                },
            },
        }));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &crate::objects::components::ObjectStableId,
        &crate::objects::components::WallMountedPlacement,
        &crate::objects::components::Wallprint,
    )>();
    let found = query.iter(world).any(|(stable_id, placement, wallprint)| {
        stable_id.0 == object_id
            && placement.attachment.height_on_wall == 0.0
            && wallprint.rects.len() == 1
            && wallprint.rects[0].height_min == 0.0
    });

    assert!(found, "door should be clamped to wall floor");
}

#[test]
fn test_wall_mounted_build_rejects_overlapping_wallprint() {
    let mut app = setup_test_app();
    let existing_id = StableObjectId(6001);
    let new_id = StableObjectId(6002);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: -1, y: -1 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(-128.0, 0.0),
        end: Vec2::new(0.0, 0.0),
        length: 128.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    let attachment = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 64.0,
        height_on_wall: 48.0,
    };
    app.world_mut().spawn((
        crate::objects::components::ObjectStableId(existing_id),
        crate::objects::components::StoreObject,
        crate::objects::components::Wallprint {
            rects: vec![crate::objects::components::derive_wallprint_rect(
                attachment,
                48.0,
                48.0,
                crate::objects::components::WallOccupancyKind::DecorativeOverlay,
            )],
        },
    ));

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id: new_id,
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: crate::objects::components::ObjectPlacement::WallMounted { attachment },
        }));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&crate::objects::components::ObjectStableId>();
    let found = query.iter(world).any(|id| id.0 == new_id);
    assert!(!found, "overlapping wall-mounted object should be rejected");
}

#[test]
fn test_wall_mounted_move_valid() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(7000);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(128.0, 0.0),
        length: 128.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    let attachment_start = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 32.0,
        height_on_wall: 48.0,
    };

    let entity = app
        .world_mut()
        .spawn((
            crate::objects::components::ObjectStableId(object_id),
            crate::objects::components::ObjectPrototypeId(BuildObjectId::new(
                "wall.decor.placeholder",
            )),
            crate::objects::components::StoreObject,
            crate::objects::components::WallMountedPlacement {
                attachment: attachment_start,
            },
            crate::objects::components::WallMounted {
                attachment: attachment_start,
                width: 64.0,
                height: 64.0,
            },
            crate::objects::components::Wallprint {
                rects: vec![crate::objects::components::derive_wallprint_rect(
                    attachment_start,
                    64.0,
                    64.0,
                    crate::objects::components::WallOccupancyKind::DecorativeOverlay,
                )],
            },
        ))
        .id();

    let attachment_end = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 96.0,
        height_on_wall: 48.0,
    };

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: attachment_end,
            },
        }));

    app.update();

    let world = app.world_mut();
    let mounted = world
        .get::<crate::objects::components::WallMountedPlacement>(entity)
        .unwrap();
    assert_eq!(
        mounted.attachment, attachment_end,
        "Object should have moved"
    );
}

#[test]
fn test_wall_mounted_move_ignores_self_overlap() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(7001);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(128.0, 0.0),
        length: 128.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    let attachment_start = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 32.0,
        height_on_wall: 48.0,
    };

    let entity = app
        .world_mut()
        .spawn((
            crate::objects::components::ObjectStableId(object_id),
            crate::objects::components::ObjectPrototypeId(BuildObjectId::new(
                "wall.decor.placeholder",
            )),
            crate::objects::components::StoreObject,
            crate::objects::components::WallMountedPlacement {
                attachment: attachment_start,
            },
            crate::objects::components::WallMounted {
                attachment: attachment_start,
                width: 64.0,
                height: 64.0,
            },
            crate::objects::components::Wallprint {
                rects: vec![crate::objects::components::derive_wallprint_rect(
                    attachment_start,
                    64.0,
                    64.0,
                    crate::objects::components::WallOccupancyKind::DecorativeOverlay,
                )],
            },
        ))
        .id();

    // Move to an overlapping position
    let attachment_end = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 40.0,
        height_on_wall: 48.0,
    };

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: attachment_end,
            },
        }));

    app.update();

    let world = app.world_mut();
    let mounted = world
        .get::<crate::objects::components::WallMountedPlacement>(entity)
        .unwrap();
    assert_eq!(
        mounted.attachment, attachment_end,
        "Object should have moved, ignoring self overlap"
    );
}

#[test]
fn test_wall_mounted_move_rejects_overlap() {
    let mut app = setup_test_app();
    let existing_id = StableObjectId(7002);
    let moving_id = StableObjectId(7003);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    app.world_mut().spawn(crate::store::WallSurface {
        key: segment_key,
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(256.0, 0.0),
        length: 256.0,
        height: 192.0,
        thickness: 8.0,
        normal: Vec2::Y,
    });

    let attachment_existing = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 128.0,
        height_on_wall: 48.0,
    };

    app.world_mut().spawn((
        crate::objects::components::ObjectStableId(existing_id),
        crate::objects::components::StoreObject,
        crate::objects::components::Wallprint {
            rects: vec![crate::objects::components::derive_wallprint_rect(
                attachment_existing,
                64.0,
                64.0,
                crate::objects::components::WallOccupancyKind::DecorativeOverlay,
            )],
        },
    ));

    let attachment_start = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 32.0,
        height_on_wall: 48.0,
    };

    let entity_moving = app
        .world_mut()
        .spawn((
            crate::objects::components::ObjectStableId(moving_id),
            crate::objects::components::ObjectPrototypeId(BuildObjectId::new(
                "wall.decor.placeholder",
            )),
            crate::objects::components::StoreObject,
            crate::objects::components::WallMountedPlacement {
                attachment: attachment_start,
            },
            crate::objects::components::WallMounted {
                attachment: attachment_start,
                width: 64.0,
                height: 64.0,
            },
            crate::objects::components::Wallprint {
                rects: vec![crate::objects::components::derive_wallprint_rect(
                    attachment_start,
                    64.0,
                    64.0,
                    crate::objects::components::WallOccupancyKind::DecorativeOverlay,
                )],
            },
        ))
        .id();

    let attachment_end = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 110.0, // Overlaps with 128.0 (width 64 means covers 96 to 160)
        height_on_wall: 48.0,
    };

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id: moving_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: attachment_end,
            },
        }));

    app.update();

    let world = app.world_mut();
    let mounted = world
        .get::<crate::objects::components::WallMountedPlacement>(entity_moving)
        .unwrap();
    assert_eq!(
        mounted.attachment, attachment_start,
        "Object should not have moved due to overlap"
    );
}

#[test]
fn test_floor_to_wall_conversion_rejected() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(7004);
    let segment_key = crate::store::WallSegmentKey {
        chunk: StoreChunkCoord { x: 0, y: 0 },
        side: crate::store::StoreBoundarySide::Top,
    };

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());

    let entity = app
        .world_mut()
        .spawn((
            crate::objects::components::ObjectStableId(object_id),
            crate::objects::components::ObjectPrototypeId(BuildObjectId::new(
                "fixture.shelf.basic",
            )),
            crate::objects::components::StoreObject,
            crate::objects::components::WorldPos(Vec2::ZERO),
            crate::objects::components::Footprint::rectangle(Vec2::splat(10.0)),
        ))
        .id();

    let attachment_end = crate::objects::components::WallAttachmentPoint {
        segment_key,
        offset_along_segment: 64.0,
        height_on_wall: 48.0,
    };

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: attachment_end,
            },
        }));

    app.update();

    let world = app.world_mut();
    assert!(
        world
            .get::<crate::objects::components::WallMountedPlacement>(entity)
            .is_none(),
        "Floor object should not be converted to WallMounted"
    );
}

#[test]
fn test_door_component_invariants_after_build() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(8000);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    queue_build_door(&mut app, object_id, segment_key, 96.0);

    app.update();

    let world = app.world_mut();
    let entity = object_entity_by_id(world, object_id).expect("door should be built");
    let entity_ref = world.entity(entity);

    assert!(entity_ref.contains::<crate::objects::components::StoreObject>());
    assert!(entity_ref.contains::<crate::objects::components::ObjectStableId>());
    assert!(entity_ref.contains::<crate::objects::components::ObjectPrototypeId>());
    assert!(entity_ref.contains::<crate::objects::components::ObjectPlacementComponent>());
    assert!(entity_ref.contains::<crate::objects::components::WallMountedPlacement>());
    assert!(entity_ref.contains::<crate::objects::components::WallMounted>());
    assert!(entity_ref.contains::<crate::objects::components::Wallprint>());
    assert!(entity_ref.contains::<crate::objects::components::Doorway>());
    assert!(entity_ref.contains::<crate::objects::components::InteriorAccessZone>());
    assert!(entity_ref.contains::<crate::objects::components::DoorMovable>());
    assert!(entity_ref.contains::<crate::objects::components::Selectable>());
    assert!(entity_ref.contains::<crate::objects::components::Inspectable>());
    assert!(entity_ref.contains::<crate::objects::components::Deletable>());
    assert!(entity_ref.contains::<crate::objects::components::Interactive>());

    assert!(!entity_ref.contains::<crate::objects::components::Footprint>());
    assert!(!entity_ref.contains::<crate::objects::components::BlocksPlacement>());
    assert!(!entity_ref.contains::<crate::objects::components::Movable>());
    assert!(!entity_ref.contains::<crate::objects::components::WallMovable>());

    let placement = entity_ref
        .get::<crate::objects::components::WallMountedPlacement>()
        .unwrap();
    assert_eq!(placement.attachment.height_on_wall, 0.0);
}

#[test]
fn test_door_move_preserves_component_invariants() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(8001);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    queue_build_door(&mut app, object_id, segment_key, 64.0);
    app.update();

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 128.0,
                    height_on_wall: 128.0,
                },
            },
        }));
    app.update();

    let world = app.world_mut();
    let entity = object_entity_by_id(world, object_id).expect("door should still exist");
    let entity_ref = world.entity(entity);
    assert!(entity_ref.contains::<crate::objects::components::Doorway>());
    assert!(entity_ref.contains::<crate::objects::components::DoorMovable>());
    assert!(entity_ref.contains::<crate::objects::components::Wallprint>());
    assert!(entity_ref.contains::<crate::objects::components::InteriorAccessZone>());
    assert!(!entity_ref.contains::<crate::objects::components::Footprint>());
    assert!(!entity_ref.contains::<crate::objects::components::BlocksPlacement>());
    assert!(!entity_ref.contains::<crate::objects::components::Movable>());
    assert!(!entity_ref.contains::<crate::objects::components::WallMovable>());

    let placement = entity_ref
        .get::<crate::objects::components::WallMountedPlacement>()
        .unwrap();
    assert_eq!(placement.attachment.offset_along_segment, 128.0);
    assert_eq!(placement.attachment.height_on_wall, 0.0);
}

#[test]
fn test_door_move_ignores_own_old_wallprint_and_access_zone() {
    let mut app = setup_test_app();
    let object_id = StableObjectId(8002);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    queue_build_door(&mut app, object_id, segment_key, 96.0);
    app.update();

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 104.0,
                    height_on_wall: 0.0,
                },
            },
        }));
    app.update();

    let world = app.world_mut();
    let entity = object_entity_by_id(world, object_id).expect("door should still exist");
    let placement = world
        .get::<crate::objects::components::WallMountedPlacement>(entity)
        .unwrap();
    assert_eq!(placement.attachment.offset_along_segment, 104.0);
}

#[test]
fn test_door_access_zone_blocks_then_moves_then_frees_floor_placement() {
    let mut app = setup_test_app();
    let door_id = StableObjectId(8003);
    let blocked_id = StableObjectId(8004);
    let old_freed_id = StableObjectId(8005);
    let new_blocked_id = StableObjectId(8006);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    queue_build_door(&mut app, door_id, segment_key, 64.0);
    app.update();

    queue_build_shelf(&mut app, blocked_id, Vec2::new(64.0, -32.0));
    app.update();
    assert!(
        !object_exists(app.world_mut(), blocked_id),
        "door access zone should block floor placement"
    );

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::MoveObject(MoveObjectCommand {
            object_id: door_id,
            new_placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 176.0,
                    height_on_wall: 0.0,
                },
            },
        }));
    app.update();

    queue_build_shelf(&mut app, old_freed_id, Vec2::new(64.0, -32.0));
    app.update();
    assert!(
        object_exists(app.world_mut(), old_freed_id),
        "old door access zone should be freed after move"
    );

    queue_build_shelf(&mut app, new_blocked_id, Vec2::new(176.0, -32.0));
    app.update();
    assert!(
        !object_exists(app.world_mut(), new_blocked_id),
        "new door access zone should block floor placement after move"
    );
}

#[test]
fn test_delete_door_frees_wallprint_and_access_zone() {
    let mut app = setup_test_app();
    let door_id = StableObjectId(8007);
    let shelf_id = StableObjectId(8008);
    let decor_id = StableObjectId(8009);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    queue_build_door(&mut app, door_id, segment_key, 96.0);
    app.update();

    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::DeleteObject(DeleteObjectCommand {
            object_id: door_id,
        }));
    app.update();

    queue_build_shelf(&mut app, shelf_id, Vec2::new(96.0, -32.0));
    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id: decor_id,
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 96.0,
                    height_on_wall: 48.0,
                },
            },
        }));
    app.update();

    assert!(!object_exists(app.world_mut(), door_id));
    assert!(
        object_exists(app.world_mut(), shelf_id),
        "floor object should be buildable in deleted door access zone"
    );
    assert!(
        object_exists(app.world_mut(), decor_id),
        "wall object should be buildable in deleted door wall interval"
    );
}

#[test]
fn test_window_and_wall_decor_do_not_create_access_zone() {
    let mut app = setup_test_app();
    let decor_id = StableObjectId(8010);
    let window_id = StableObjectId(8011);
    let segment_key = test_segment_key();

    app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());
    spawn_test_wall_surface(&mut app, segment_key);
    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id: decor_id,
            prototype_id: BuildObjectId::new("wall.decor.placeholder"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 48.0,
                    height_on_wall: 48.0,
                },
            },
        }));
    app.world_mut()
        .resource_mut::<DomainCommandQueue>()
        .commands
        .push_back(DomainCommand::BuildObject(BuildObjectCommand {
            object_id: window_id,
            prototype_id: BuildObjectId::new("wall.window.basic_visual"),
            placement: crate::objects::components::ObjectPlacement::WallMounted {
                attachment: crate::objects::components::WallAttachmentPoint {
                    segment_key,
                    offset_along_segment: 144.0,
                    height_on_wall: 48.0,
                },
            },
        }));
    app.update();

    let world = app.world_mut();
    for object_id in [decor_id, window_id] {
        let entity = object_entity_by_id(world, object_id).expect("wall object should be built");
        assert!(
            !world
                .entity(entity)
                .contains::<crate::objects::components::InteriorAccessZone>(),
            "non-door wall-mounted objects must not reserve interior access zones"
        );
    }
}
