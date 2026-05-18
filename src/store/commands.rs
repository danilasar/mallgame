use crate::objects::components::{ObjectPlacement, StableObjectId, WallMountedBounds};
use crate::objects::prototypes::{BuildObjectId, ObjectCatalog, wall_mounted_spec};
use crate::store::chunks::{StoreChunkCoord, StoreChunkKind};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Resource, Default, Debug)]
pub struct DomainCommandQueue {
    pub commands: VecDeque<DomainCommand>,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainCommandSet {
    RequestToCommand,
    ApplyCommands,
    EmitEvents,
    PostDomainApply,
}

#[derive(Debug, Clone)]
pub enum DomainCommand {
    BuildObject(BuildObjectCommand),
    MoveObject(MoveObjectCommand),
    RotateObject(RotateObjectCommand),
    DeleteObject(DeleteObjectCommand),
    PurchaseChunk(PurchaseChunkCommand),
}

#[derive(Debug, Clone)]
pub struct BuildObjectCommand {
    pub object_id: StableObjectId,
    pub prototype_id: BuildObjectId,
    pub placement: ObjectPlacement,
}

#[derive(Debug, Clone)]
pub struct MoveObjectCommand {
    pub object_id: StableObjectId,
    pub from: Vec2,
    pub to: Vec2,
    pub rotation_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RotateObjectCommand {
    pub object_id: StableObjectId,
    pub from_rotation: usize,
    pub to_rotation: usize,
}

#[derive(Debug, Clone)]
pub struct DeleteObjectCommand {
    pub object_id: StableObjectId,
}

#[derive(Debug, Clone)]
pub struct PurchaseChunkCommand {
    pub coord: StoreChunkCoord,
    pub kind: StoreChunkKind,
}

#[derive(Debug, Clone)]
pub enum DomainCommandResult {
    Applied {
        events: Vec<crate::store::events::DomainEvent>,
    },
    Rejected {
        reason: DomainCommandRejectReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainCommandRejectReason {
    ObjectMissing {
        id: StableObjectId,
    },
    #[allow(dead_code)]
    ObjectAlreadyExists {
        id: StableObjectId,
    },
    #[allow(dead_code)]
    PrototypeMissing {
        id: BuildObjectId,
    },
    PlacementInvalid,
    PrototypeNotWallMounted,
    WallSegmentMissing,
    WallSideNotAllowed,
    WallOffsetOutOfBounds,
    WallHeightOutOfBounds,
    WallMountedOverlap,
    RotationInvalid,
    ChunkInvalid,
    #[allow(dead_code)]
    DirectionNotAllowed,
    #[allow(dead_code)]
    WouldCreateHole,
    #[allow(dead_code)]
    NotSideAdjacent,
    #[allow(dead_code)]
    OutsideWorldBounds,
    #[allow(dead_code)]
    AlreadyOwned,
}

#[derive(Message, Debug, Clone)]
#[allow(dead_code)]
pub struct DomainCommandRejected {
    pub command: DomainCommand,
    pub reason: DomainCommandRejectReason,
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub struct DomainCommandApplyParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub queue: ResMut<'w, DomainCommandQueue>,
    pub events: MessageWriter<'w, crate::store::events::DomainEvent>,
    pub rejections: MessageWriter<'w, DomainCommandRejected>,
    pub asset_server: Res<'w, AssetServer>,
    pub world_bounds: Res<'w, crate::store::WorldBounds>,
    pub catalog: Res<'w, ObjectCatalog>,
    pub store: ResMut<'w, crate::store::area::StoreArea>,
    pub _allocator: ResMut<'w, crate::objects::components::StableObjectIdAllocator>,
    pub set: ParamSet<
        'w,
        's,
        (
            Query<
                'w,
                's,
                (
                    Entity,
                    &'static crate::objects::components::WorldPos,
                    &'static crate::objects::components::Footprint,
                    Option<&'static crate::objects::components::BlocksPlacement>,
                ),
                Without<crate::objects::components::WallMounted>,
            >,
            Query<
                'w,
                's,
                (
                    &'static mut crate::objects::rotation::Rotatable,
                    &'static mut Sprite,
                    &'static mut crate::objects::components::Footprint,
                    &'static mut crate::objects::components::FootAnchor,
                    &'static mut crate::objects::components::VisualOffset,
                ),
                Without<crate::tools::ToolPreview>,
            >,
            Query<'w, 's, (Entity, &'static crate::objects::components::ObjectStableId)>,
            Query<'w, 's, &'static crate::store::WallSurface>,
            Query<'w, 's, &'static WallMountedBounds>,
        ),
    >,
}

pub fn apply_domain_commands(params: DomainCommandApplyParams) {
    let DomainCommandApplyParams {
        mut commands,
        mut queue,
        mut events,
        mut rejections,
        asset_server,
        world_bounds,
        catalog,
        mut store,
        _allocator,
        mut set,
    } = params;

    let mut lookup = std::collections::HashMap::new();
    for (entity, stable_id) in &set.p2() {
        lookup.insert(stable_id.0, entity);
    }

    while let Some(command) = queue.commands.pop_front() {
        let result = match &command {
            DomainCommand::BuildObject(c) => apply_build_object(
                c,
                &mut commands,
                &asset_server,
                &catalog,
                &world_bounds,
                &store,
                &mut set,
            ),
            DomainCommand::MoveObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_move_object(c, entity, &mut commands, &mut set, &world_bounds, &store)
                } else {
                    DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                    }
                }
            }
            DomainCommand::RotateObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_rotate_object(c, entity, &mut set, &world_bounds, &store)
                } else {
                    DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                    }
                }
            }
            DomainCommand::DeleteObject(c) => apply_delete_object(c, &mut commands, &lookup),
            DomainCommand::PurchaseChunk(c) => apply_purchase_chunk(c, &world_bounds, &mut store),
        };

        match result {
            DomainCommandResult::Applied { events: new_events } => {
                for event in new_events {
                    events.write(event);
                }
            }
            DomainCommandResult::Rejected { reason } => {
                warn!("Command rejected: {:?} Reason: {:?}", command, reason);
                rejections.write(DomainCommandRejected { command, reason });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_build_object(
    c: &BuildObjectCommand,
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &ObjectCatalog,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
    set: &mut ParamSet<(
        Query<
            (
                Entity,
                &crate::objects::components::WorldPos,
                &crate::objects::components::Footprint,
                Option<&crate::objects::components::BlocksPlacement>,
            ),
            Without<crate::objects::components::WallMounted>,
        >,
        Query<
            (
                &mut crate::objects::rotation::Rotatable,
                &mut Sprite,
                &mut crate::objects::components::Footprint,
                &mut crate::objects::components::FootAnchor,
                &mut crate::objects::components::VisualOffset,
            ),
            Without<crate::tools::ToolPreview>,
        >,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
        Query<&crate::store::WallSurface>,
        Query<&WallMountedBounds>,
    )>,
) -> DomainCommandResult {
    let Some(proto) = catalog.prototypes.get(&c.prototype_id) else {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::PrototypeMissing {
                id: c.prototype_id.clone(),
            },
        };
    };

    match c.placement {
        ObjectPlacement::Floor { world_pos, .. } => {
            if proto.placement.kind != crate::objects::prototypes::PlacementKind::Floor {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PlacementInvalid,
                };
            }

            let footprint = crate::objects::components::Footprint::rectangle(
                proto.placement.footprint_half_extents,
            );

            let footprints = set.p0();
            let validation = crate::placement::validate_placement(
                world_bounds,
                store,
                &footprints,
                &footprint,
                world_pos,
                crate::placement::PlacementValidationOptions::default(),
            );

            if validation.is_err() {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PlacementInvalid,
                };
            }
        }
        ObjectPlacement::WallMounted { attachment } => {
            if proto.placement.kind != crate::objects::prototypes::PlacementKind::WallMounted {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PrototypeNotWallMounted,
                };
            }
            let Some(spec) = wall_mounted_spec(proto) else {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PrototypeNotWallMounted,
                };
            };
            let surface = {
                let wall_surfaces = set.p3();
                let Some(surface) = wall_surfaces
                    .iter()
                    .find(|surface| surface.key == attachment.segment_key)
                    .copied()
                else {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::WallSegmentMissing,
                    };
                };
                surface
            };
            if !spec.allowed_sides.contains(&attachment.segment_key.side) {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::WallSideNotAllowed,
                };
            }
            let half_width = spec.width * 0.5;
            if attachment.offset_along_segment < half_width
                || attachment.offset_along_segment > surface.length - half_width
            {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::WallOffsetOutOfBounds,
                };
            }
            if attachment.height_on_wall < 0.0
                || attachment.height_on_wall > surface.height - spec.height
            {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::WallHeightOutOfBounds,
                };
            }
            let mounted_objects = set.p4();
            let new_min = attachment.offset_along_segment - half_width;
            let new_max = attachment.offset_along_segment + half_width;
            let new_height_min = attachment.height_on_wall;
            let new_height_max = attachment.height_on_wall + spec.height;
            for existing in mounted_objects.iter() {
                if existing.segment_key != attachment.segment_key {
                    continue;
                }
                let horizontal_overlap =
                    new_min < existing.offset_max && new_max > existing.offset_min;
                let vertical_overlap =
                    new_height_min < existing.height_max && new_height_max > existing.height_min;
                if horizontal_overlap && vertical_overlap {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::WallMountedOverlap,
                    };
                }
            }
        }
    }

    if let Err(e) = crate::objects::prototypes::spawn_store_object_from_prototype(
        commands,
        asset_server,
        catalog,
        crate::objects::prototypes::SpawnStoreObjectParams {
            stable_id: c.object_id,
            prototype_id: c.prototype_id.clone(),
            placement: c.placement,
        },
    ) {
        error!("Spawn failed: {}", e);
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::PlacementInvalid,
        }; // Fallback
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectBuilt { id: c.object_id }],
    }
}

#[allow(clippy::type_complexity)]
fn apply_move_object(
    c: &MoveObjectCommand,
    entity: Entity,
    commands: &mut Commands,
    set: &mut ParamSet<(
        Query<
            (
                Entity,
                &crate::objects::components::WorldPos,
                &crate::objects::components::Footprint,
                Option<&crate::objects::components::BlocksPlacement>,
            ),
            Without<crate::objects::components::WallMounted>,
        >,
        Query<
            (
                &mut crate::objects::rotation::Rotatable,
                &mut Sprite,
                &mut crate::objects::components::Footprint,
                &mut crate::objects::components::FootAnchor,
                &mut crate::objects::components::VisualOffset,
            ),
            Without<crate::tools::ToolPreview>,
        >,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
        Query<&crate::store::WallSurface>,
        Query<&WallMountedBounds>,
    )>,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
) -> DomainCommandResult {
    // 1. Get current footprint for validation
    let footprints = set.p0();
    let Ok((_, _, footprint, _)) = footprints.get(entity) else {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
        };
    };
    let current_footprint = footprint.clone();

    // 2. Revalidate position
    let validation = crate::placement::validate_placement(
        world_bounds,
        store,
        &footprints,
        &current_footprint,
        c.to,
        crate::placement::PlacementValidationOptions {
            ignore_entity: Some(entity),
        },
    );

    if let Err(_e) = validation {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::PlacementInvalid,
        };
    }

    // 3. Apply changes (Pos + Rotation)
    if let Ok(mut e) = commands.get_entity(entity) {
        e.insert(crate::objects::components::WorldPos(c.to));
    }

    if let Some(new_rotation) = c.rotation_index {
        let mut rotatables = set.p1();
        if let Ok((mut rotatable, mut sprite, mut fp, mut anchor, mut offset)) =
            rotatables.get_mut(entity)
            && new_rotation < rotatable.variants.len()
        {
            rotatable.current = new_rotation;
            let variant = &rotatable.variants[new_rotation];
            sprite.image = variant.sprite.clone();
            *fp = variant.footprint.clone();
            anchor.0 = variant.foot_anchor;
            offset.0 = variant.visual_offset;
        }
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectMoved {
            id: c.object_id,
            from: c.from,
            to: c.to,
        }],
    }
}

#[allow(clippy::type_complexity)]
fn apply_rotate_object(
    c: &RotateObjectCommand,
    entity: Entity,
    set: &mut ParamSet<(
        Query<
            (
                Entity,
                &crate::objects::components::WorldPos,
                &crate::objects::components::Footprint,
                Option<&crate::objects::components::BlocksPlacement>,
            ),
            Without<crate::objects::components::WallMounted>,
        >,
        Query<
            (
                &mut crate::objects::rotation::Rotatable,
                &mut Sprite,
                &mut crate::objects::components::Footprint,
                &mut crate::objects::components::FootAnchor,
                &mut crate::objects::components::VisualOffset,
            ),
            Without<crate::tools::ToolPreview>,
        >,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
        Query<&crate::store::WallSurface>,
        Query<&WallMountedBounds>,
    )>,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
) -> DomainCommandResult {
    // 1. Get rotation variant and position
    let (variant, world_pos) = {
        let q1 = set.p1();
        match q1.get(entity) {
            Ok((rotatable, _, _, _, _)) => {
                if c.to_rotation >= rotatable.variants.len() {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::RotationInvalid,
                    };
                }
                let variant = rotatable.variants[c.to_rotation].clone();

                let q0 = set.p0();
                match q0.get(entity) {
                    Ok((_, pos, _, _)) => (variant, pos.0),
                    Err(_) => {
                        return DomainCommandResult::Rejected {
                            reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                        };
                    }
                }
            }
            Err(_) => {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::RotationInvalid,
                };
            }
        }
    };

    // 2. Validate rotated placement
    let validation = {
        let footprints = set.p0();
        crate::placement::validate_placement(
            world_bounds,
            store,
            &footprints,
            &variant.footprint,
            world_pos,
            crate::placement::PlacementValidationOptions {
                ignore_entity: Some(entity),
            },
        )
    };

    if let Err(_e) = validation {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::RotationInvalid,
        };
    }

    // 3. Apply mutation
    let mut q1 = set.p1();
    if let Ok((mut rotatable, mut sprite, mut fp, mut anchor, mut offset)) = q1.get_mut(entity) {
        rotatable.current = c.to_rotation;
        sprite.image = variant.sprite;
        *fp = variant.footprint;
        anchor.0 = variant.foot_anchor;
        offset.0 = variant.visual_offset;

        DomainCommandResult::Applied {
            events: vec![crate::store::events::DomainEvent::ObjectRotated {
                id: c.object_id,
                from: c.from_rotation,
                to: c.to_rotation,
            }],
        }
    } else {
        DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::RotationInvalid,
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_delete_object(
    c: &DeleteObjectCommand,
    commands: &mut Commands,
    lookup: &std::collections::HashMap<StableObjectId, Entity>,
) -> DomainCommandResult {
    let Some(&entity) = lookup.get(&c.object_id) else {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
        };
    };

    commands.entity(entity).try_despawn();

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectDeleted { id: c.object_id }],
    }
}

#[allow(clippy::type_complexity)]
fn apply_purchase_chunk(
    c: &PurchaseChunkCommand,
    world_bounds: &crate::store::WorldBounds,
    store: &mut crate::store::area::StoreArea,
) -> DomainCommandResult {
    let validation =
        crate::store::expansion::validate_chunk_purchase(world_bounds, store, c.coord, c.kind);
    if !validation.valid {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::ChunkInvalid,
        };
    }

    store.owned_chunks.insert(
        c.coord,
        crate::store::chunks::StoreChunkData { kind: c.kind },
    );

    DomainCommandResult::Applied {
        events: vec![
            crate::store::events::DomainEvent::ChunkPurchased { coord: c.coord },
            crate::store::events::DomainEvent::StoreAreaChanged {
                added_chunks: vec![c.coord],
                removed_chunks: vec![],
            },
        ],
    }
}

#[cfg(test)]
pub fn register_test_messages(app: &mut App) {
    app.add_message::<crate::store::events::DomainEvent>()
        .add_message::<DomainCommandRejected>()
        .add_message::<crate::tools::ObjectActionRequested>()
        .add_message::<crate::tools::MoveObjectCommitted>()
        .add_message::<crate::tools::DeleteObjectRequested>()
        .add_message::<crate::tools::BuildObjectRequested>()
        .add_message::<crate::tools::StartMoveObjectRequested>()
        .add_message::<crate::tools::ToolChangedRequested>()
        .add_message::<crate::tools::ActivateToolRequested>()
        .add_message::<crate::tools::ReturnToPreviousToolRequested>()
        .add_message::<crate::objects::rotation::RotateObjectRequested>();
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_delete_object_command() {
        let mut app = setup_test_app();
        let object_id = StableObjectId(102);

        app.add_systems(Update, (apply_domain_commands, ApplyDeferred).chain());

        app.world_mut().spawn((
            crate::objects::components::ObjectStableId(object_id),
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
            &crate::objects::components::WallMountedBounds,
            Option<&crate::objects::components::Footprint>,
            Option<&crate::objects::components::BlocksPlacement>,
            Option<&crate::objects::components::Movable>,
            &crate::objects::components::StoreObject,
        )>();
        let found = query.iter(world).any(
            |(stable_id, mounted, bounds, footprint, blocker, movable, _store_object)| {
                stable_id.0 == object_id
                    && mounted.attachment.segment_key == segment_key
                    && bounds.segment_key == segment_key
                    && footprint.is_none()
                    && blocker.is_none()
                    && movable.is_none()
            },
        );

        assert!(found, "wall-mounted StoreObject was not spawned correctly");
    }
}
