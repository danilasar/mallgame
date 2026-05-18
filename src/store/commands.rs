use bevy::prelude::*;
use crate::objects::components::StableObjectId;
use crate::objects::prototypes::BuildPrototypeId;
use crate::store::chunks::{StoreChunkCoord, StoreChunkKind};
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
    pub prototype_id: BuildPrototypeId,
    pub world_pos: Vec2,
    pub rotation_index: Option<usize>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        id: BuildPrototypeId,
    },
    PlacementInvalid,
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
pub struct DomainCommandRejected {
    pub command: DomainCommand,
    pub reason: DomainCommandRejectReason,
}

pub fn apply_domain_commands(
    mut commands: Commands,
    mut queue: ResMut<DomainCommandQueue>,
    mut events: MessageWriter<crate::store::events::DomainEvent>,
    mut rejections: MessageWriter<DomainCommandRejected>,
    asset_server: Res<AssetServer>,
    world_bounds: Res<crate::store::WorldBounds>,
    mut store: ResMut<crate::store::area::StoreArea>,
    mut _allocator: ResMut<crate::objects::components::StableObjectIdAllocator>,
    mut set: ParamSet<(
        Query<(Entity, &crate::objects::components::WorldPos, &crate::objects::components::Footprint, Option<&crate::objects::components::BlocksPlacement>)>,
        Query<(&mut crate::objects::rotation::Rotatable, &mut Sprite, &mut crate::objects::components::Footprint, &mut crate::objects::components::FootAnchor, &mut crate::objects::components::VisualOffset), Without<crate::tools::ToolPreview>>,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
    )>,
) {
    let mut lookup = std::collections::HashMap::new();
    for (entity, stable_id) in &set.p2() {
        lookup.insert(stable_id.0, entity);
    }

    while let Some(command) = queue.commands.pop_front() {
        let result = match &command {
            DomainCommand::BuildObject(c) => apply_build_object(c, &mut commands, &asset_server, &world_bounds, &store, &set.p0()),
            DomainCommand::MoveObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_move_object(c, entity, &mut commands, &mut set, &world_bounds, &store)
                } else {
                    DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } }
                }
            }
            DomainCommand::RotateObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_rotate_object(c, entity, &mut set, &world_bounds, &store)
                } else {
                    DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } }
                }
            },
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

fn apply_build_object(
    c: &BuildObjectCommand,
    commands: &mut Commands,
    asset_server: &AssetServer,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
    footprints: &Query<(Entity, &crate::objects::components::WorldPos, &crate::objects::components::Footprint, Option<&crate::objects::components::BlocksPlacement>)>,
) -> DomainCommandResult {
    let spec = crate::objects::prototypes::prototype_spec(c.prototype_id);
    let footprint = crate::objects::components::Footprint::rectangle(spec.footprint_half_extents);

    let validation = crate::placement::validate_placement(
        world_bounds,
        store,
        footprints,
        &footprint,
        c.world_pos,
        crate::placement::PlacementValidationOptions::default(),
    );

    if let Err(_e) = validation {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::PlacementInvalid };
    }

    crate::objects::prototypes::spawn_store_object_from_prototype(
        commands,
        asset_server,
        crate::objects::prototypes::SpawnStoreObjectParams {
            stable_id: c.object_id,
            prototype_id: c.prototype_id,
            world_pos: c.world_pos,
            rotation_index: c.rotation_index,
        },
    );

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectBuilt { id: c.object_id }],
    }
}

fn apply_move_object(
    c: &MoveObjectCommand,
    entity: Entity,
    commands: &mut Commands,
    set: &mut ParamSet<(
        Query<(Entity, &crate::objects::components::WorldPos, &crate::objects::components::Footprint, Option<&crate::objects::components::BlocksPlacement>)>,
        Query<(&mut crate::objects::rotation::Rotatable, &mut Sprite, &mut crate::objects::components::Footprint, &mut crate::objects::components::FootAnchor, &mut crate::objects::components::VisualOffset), Without<crate::tools::ToolPreview>>,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
    )>,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
) -> DomainCommandResult {
    // 1. Get current footprint for validation
    let footprints = set.p0();
    let Ok((_, _, footprint, _)) = footprints.get(entity) else {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } };
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
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::PlacementInvalid };
    }

    // 3. Apply changes (Pos + Rotation)
    if let Ok(mut e) = commands.get_entity(entity) {
        e.insert(crate::objects::components::WorldPos(c.to));
    }

    if let Some(new_rotation) = c.rotation_index {
        let mut rotatables = set.p1();
        if let Ok((mut rotatable, mut sprite, mut fp, mut anchor, mut offset)) = rotatables.get_mut(entity) {
            if new_rotation < rotatable.variants.len() {
                rotatable.current = new_rotation;
                let variant = &rotatable.variants[new_rotation];
                sprite.image = variant.sprite.clone();
                *fp = variant.footprint.clone();
                anchor.0 = variant.foot_anchor;
                offset.0 = variant.visual_offset;
            }
        }
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectMoved { id: c.object_id, from: c.from, to: c.to }],
    }
}

fn apply_rotate_object(
    c: &RotateObjectCommand,
    entity: Entity,
    set: &mut ParamSet<(
        Query<(Entity, &crate::objects::components::WorldPos, &crate::objects::components::Footprint, Option<&crate::objects::components::BlocksPlacement>)>,
        Query<(&mut crate::objects::rotation::Rotatable, &mut Sprite, &mut crate::objects::components::Footprint, &mut crate::objects::components::FootAnchor, &mut crate::objects::components::VisualOffset), Without<crate::tools::ToolPreview>>,
        Query<(Entity, &crate::objects::components::ObjectStableId)>,
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
                    return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid };
                }
                let variant = rotatable.variants[c.to_rotation].clone();
                
                let q0 = set.p0();
                match q0.get(entity) {
                    Ok((_, pos, _, _)) => (variant, pos.0),
                    Err(_) => return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } },
                }
            }
            Err(_) => return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid },
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
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid };
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
            events: vec![crate::store::events::DomainEvent::ObjectRotated { id: c.object_id, from: c.from_rotation, to: c.to_rotation }],
        }
    } else {
        DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid }
    }
}

fn apply_delete_object(
    c: &DeleteObjectCommand,
    commands: &mut Commands,
    lookup: &std::collections::HashMap<StableObjectId, Entity>,
) -> DomainCommandResult {
    let Some(&entity) = lookup.get(&c.object_id) else {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } };
    };

    if let Ok(mut e) = commands.get_entity(entity) {
        e.despawn();
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectDeleted { id: c.object_id }],
    }
}

fn apply_purchase_chunk(
    c: &PurchaseChunkCommand,
    world_bounds: &crate::store::WorldBounds,
    store: &mut crate::store::area::StoreArea,
) -> DomainCommandResult {
    let validation = crate::store::expansion::validate_chunk_purchase(world_bounds, store, c.coord, c.kind);
    if !validation.valid {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ChunkInvalid };
    }

    store.owned_chunks.insert(c.coord, crate::store::chunks::StoreChunkData { kind: c.kind });

    DomainCommandResult::Applied {
        events: vec![
            crate::store::events::DomainEvent::ChunkPurchased { coord: c.coord },
            crate::store::events::DomainEvent::StoreAreaChanged { added_chunks: vec![c.coord], removed_chunks: vec![] },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::components::InteractionRole;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_resource::<DomainCommandQueue>();
        app.add_message::<crate::store::events::DomainEvent>();
        app.add_message::<DomainCommandRejected>();
        app.init_resource::<crate::store::WorldBounds>();
        app.insert_resource(crate::store::area::StoreArea::new(Vec2::ZERO));
        app.insert_resource(crate::objects::components::StableObjectIdAllocator { next: 1000 });
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

        app.world_mut().resource_mut::<DomainCommandQueue>().commands.push_back(
            DomainCommand::DeleteObject(DeleteObjectCommand { object_id })
        );

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

        app.world_mut().resource_mut::<DomainCommandQueue>().commands.push_back(
            DomainCommand::BuildObject(BuildObjectCommand {
                object_id: id2,
                prototype_id: BuildPrototypeId::Chair,
                world_pos: pos,
                rotation_index: Some(0),
            })
        );

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&crate::objects::components::ObjectStableId>();
        let found = query.iter(world).any(|id| id.0 == id2);
        assert!(!found, "Object with ID 202 should NOT have been built due to collision");
    }
}
