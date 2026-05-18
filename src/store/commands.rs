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
            DomainCommand::MoveObject(c) => apply_move_object(c, &mut commands, &world_bounds, &store, &set.p0(), &lookup),
            DomainCommand::RotateObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    let (variant, world_pos) = {
                        let q1 = set.p1();
                        match q1.get(entity) {
                            Ok((rotatable, _, _, _, _)) => {
                                if c.to_rotation >= rotatable.variants.len() {
                                    (None, Vec2::ZERO)
                                } else {
                                    let variant = rotatable.variants[c.to_rotation].clone();
                                    let q0 = set.p0();
                                    match q0.get(entity) {
                                        Ok((_, pos, _, _)) => (Some(variant), pos.0),
                                        Err(_) => (None, Vec2::ZERO),
                                    }
                                }
                            }
                            Err(_) => (None, Vec2::ZERO),
                        }
                    };

                    if let Some(variant) = variant {
                        let validation = crate::placement::validate_placement(
                            &world_bounds,
                            &store,
                            &set.p0(),
                            &variant.footprint,
                            world_pos,
                            crate::placement::PlacementValidationOptions {
                                ignore_entity: Some(entity),
                            },
                        );

                        if validation.is_err() {
                            DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid }
                        } else {
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
                    } else {
                        DomainCommandResult::Rejected { reason: DomainCommandRejectReason::RotationInvalid }
                    }
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
    commands: &mut Commands,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
    footprints: &Query<(Entity, &crate::objects::components::WorldPos, &crate::objects::components::Footprint, Option<&crate::objects::components::BlocksPlacement>)>,
    lookup: &std::collections::HashMap<StableObjectId, Entity>,
) -> DomainCommandResult {
    let Some(&entity) = lookup.get(&c.object_id) else {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } };
    };

    let Ok((_, _, footprint, _)) = footprints.get(entity) else {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id } };
    };

    let validation = crate::placement::validate_placement(
        world_bounds,
        store,
        footprints,
        footprint,
        c.to,
        crate::placement::PlacementValidationOptions {
            ignore_entity: Some(entity),
        },
    );

    if let Err(_e) = validation {
        return DomainCommandResult::Rejected { reason: DomainCommandRejectReason::PlacementInvalid };
    }

    if let Ok(mut e) = commands.get_entity(entity) {
        e.insert(crate::objects::components::WorldPos(c.to));
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectMoved { id: c.object_id, from: c.from, to: c.to }],
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
