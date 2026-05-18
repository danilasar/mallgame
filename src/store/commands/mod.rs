#[cfg(test)]
mod tests;

use crate::objects::components::{
    InteriorAccessZone, ObjectPlacement, ObjectStableId, StableObjectId, Wallprint,
    derive_wallprint, wallprints_conflict,
};
use crate::objects::prototypes::{
    BuildObjectId, ObjectCatalog, wall_mounted_spec, wall_occupancy_kind_for_prototype,
};
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
    pub new_placement: ObjectPlacement,
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
    pub wallprints: Query<
        'w,
        's,
        (
            &'static Wallprint,
            &'static crate::objects::components::ObjectStableId,
        ),
    >,
    pub access_zones: Query<
        'w,
        's,
        (
            Entity,
            &'static InteriorAccessZone,
            Option<&'static ObjectStableId>,
        ),
    >,
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
            Query<
                'w,
                's,
                (
                    Entity,
                    &'static crate::objects::components::ObjectStableId,
                    &'static crate::objects::components::ObjectPrototypeId,
                ),
            >,
            Query<'w, 's, &'static crate::store::WallSurface>,
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
        wallprints,
        access_zones,
        mut set,
    } = params;

    let mut lookup = std::collections::HashMap::new();
    for (entity, stable_id, _) in &set.p2() {
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
                &wallprints,
                &access_zones,
                &mut set,
            ),
            DomainCommand::MoveObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_move_object(
                        c,
                        entity,
                        &mut commands,
                        &mut set,
                        &wallprints,
                        &access_zones,
                        &world_bounds,
                        &store,
                        &catalog,
                    )
                } else {
                    DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                    }
                }
            }
            DomainCommand::RotateObject(c) => {
                if let Some(&entity) = lookup.get(&c.object_id) {
                    apply_rotate_object(c, entity, &mut set, &access_zones, &world_bounds, &store)
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
#[allow(clippy::too_many_arguments)]
fn apply_build_object(
    c: &BuildObjectCommand,
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &ObjectCatalog,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
    wallprints: &Query<(&Wallprint, &crate::objects::components::ObjectStableId)>,
    access_zones: &Query<(Entity, &InteriorAccessZone, Option<&ObjectStableId>)>,
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
        Query<(
            Entity,
            &crate::objects::components::ObjectStableId,
            &crate::objects::components::ObjectPrototypeId,
        )>,
        Query<&crate::store::WallSurface>,
    )>,
) -> DomainCommandResult {
    let Some(proto) = catalog.prototypes.get(&c.prototype_id) else {
        return DomainCommandResult::Rejected {
            reason: DomainCommandRejectReason::PrototypeMissing {
                id: c.prototype_id.clone(),
            },
        };
    };

    let mut derived_door = None;
    let mut effective_placement = c.placement;

    match effective_placement {
        ObjectPlacement::Floor { world_pos, .. } => {
            if proto.placement.kind != crate::objects::prototypes::PlacementKind::Floor {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PlacementInvalid,
                };
            }

            let footprint = crate::objects::components::Footprint::rectangle(
                proto.placement.footprint_half_extents,
            );

            let validation = {
                let footprints = set.p0();
                crate::placement::validate_placement(
                    world_bounds,
                    store,
                    &footprints,
                    access_zones,
                    &footprint,
                    world_pos,
                    crate::placement::PlacementValidationOptions::default(),
                )
            };

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
            let attachment = crate::objects::prototypes::normalize_wall_attachment_for_prototype(
                proto, attachment,
            );
            effective_placement = ObjectPlacement::WallMounted { attachment };
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

            if let Some(door_spec) = crate::objects::prototypes::doorway_spec(proto) {
                let derived = crate::store::boundary::derive_door_placement(
                    spec.width,
                    spec.height,
                    door_spec.access_width,
                    door_spec.access_depth,
                    attachment,
                    &surface,
                    wall_occupancy_kind_for_prototype(proto),
                );

                let derived = match derived {
                    Ok(d) => d,
                    Err(_) => {
                        return DomainCommandResult::Rejected {
                            reason: DomainCommandRejectReason::PlacementInvalid,
                        };
                    }
                };

                let validation = {
                    let footprints = set.p0();
                    crate::placement::validate_derived_door_placement(
                        &derived,
                        store,
                        &footprints,
                        wallprints,
                        crate::placement::PlacementValidationOptions::default(),
                    )
                };

                if validation.is_err() {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::PlacementInvalid,
                    };
                }
                derived_door = Some(derived);
            } else {
                let new_wallprint = derive_wallprint(
                    attachment,
                    spec.width,
                    spec.height,
                    wall_occupancy_kind_for_prototype(proto),
                );
                for (existing, _) in wallprints.iter() {
                    if wallprints_conflict(&new_wallprint, existing) {
                        return DomainCommandResult::Rejected {
                            reason: DomainCommandRejectReason::WallMountedOverlap,
                        };
                    }
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
            placement: effective_placement,
            derived_door,
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
#[allow(clippy::too_many_arguments)]
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
        Query<(
            Entity,
            &crate::objects::components::ObjectStableId,
            &crate::objects::components::ObjectPrototypeId,
        )>,
        Query<&crate::store::WallSurface>,
    )>,
    wallprints: &Query<(&Wallprint, &crate::objects::components::ObjectStableId)>,
    access_zones: &Query<(Entity, &InteriorAccessZone, Option<&ObjectStableId>)>,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
    catalog: &crate::objects::prototypes::ObjectCatalog,
) -> DomainCommandResult {
    match c.new_placement {
        ObjectPlacement::Floor {
            world_pos,
            rotation_index,
        } => {
            let stable_id = {
                let stable_ids = set.p2();
                let Ok((_, sid, _)) = stable_ids.get(entity) else {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                    };
                };
                sid.0
            };
            // 1. Get current footprint for validation
            let footprints = set.p0();
            let Ok((_, _, footprint, _)) = footprints.get(entity) else {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PlacementInvalid,
                };
            };
            let current_footprint = footprint.clone();

            // 2. Revalidate position
            let validation = {
                let footprints = set.p0();
                crate::placement::validate_placement(
                    world_bounds,
                    store,
                    &footprints,
                    access_zones,
                    &current_footprint,
                    world_pos,
                    crate::placement::PlacementValidationOptions {
                        ignore_entity: Some(entity),
                        ignore_stable_id: Some(stable_id),
                    },
                )
            };

            if let Err(_e) = validation {
                return DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PlacementInvalid,
                };
            }

            // 3. Apply changes (Pos + Rotation)
            if let Ok(mut e) = commands.get_entity(entity) {
                e.insert(crate::objects::components::WorldPos(world_pos));
                e.insert(crate::objects::components::ObjectPlacementComponent {
                    placement: c.new_placement,
                });
            }

            if let Some(new_rotation) = rotation_index {
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
        }
        ObjectPlacement::WallMounted { attachment } => {
            // 1. Verify object is wall-mounted and get its prototype
            let (prototype_id, stable_id) = {
                let stable_ids = set.p2();
                let Ok((_, sid, pid)) = stable_ids.get(entity) else {
                    return DomainCommandResult::Rejected {
                        reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
                    };
                };
                (pid.0.clone(), sid.0)
            };

            let proto = catalog.prototypes.get(&prototype_id).ok_or({
                DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PrototypeMissing { id: prototype_id },
                }
            });

            let proto = match proto {
                Ok(p) => p,
                Err(e) => return e,
            };

            let spec = crate::objects::prototypes::wall_mounted_spec(proto).ok_or({
                DomainCommandResult::Rejected {
                    reason: DomainCommandRejectReason::PrototypeNotWallMounted,
                }
            });

            let spec = match spec {
                Ok(s) => s,
                Err(e) => return e,
            };
            let attachment = crate::objects::prototypes::normalize_wall_attachment_for_prototype(
                proto, attachment,
            );
            let effective_placement = ObjectPlacement::WallMounted { attachment };

            // 2. Derive new wallprint
            let occupancy_kind =
                crate::objects::prototypes::wall_occupancy_kind_for_prototype(proto);

            if let Some(door_spec) = crate::objects::prototypes::doorway_spec(proto) {
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

                let derived = crate::store::boundary::derive_door_placement(
                    spec.width,
                    spec.height,
                    door_spec.access_width,
                    door_spec.access_depth,
                    attachment,
                    &surface,
                    occupancy_kind,
                );

                let derived = match derived {
                    Ok(d) => d,
                    Err(_) => {
                        return DomainCommandResult::Rejected {
                            reason: DomainCommandRejectReason::PlacementInvalid,
                        };
                    }
                };

                // 3. Validate against other wallprints and footprints (ignoring self)
                {
                    let footprints = set.p0();
                    let validation = crate::placement::validate_derived_door_placement(
                        &derived,
                        store,
                        &footprints,
                        wallprints,
                        crate::placement::PlacementValidationOptions {
                            ignore_entity: Some(entity),
                            ignore_stable_id: Some(stable_id),
                        },
                    );

                    if validation.is_err() {
                        return DomainCommandResult::Rejected {
                            reason: DomainCommandRejectReason::PlacementInvalid,
                        };
                    }
                }

                // 4. Update components
                if let Ok(mut e) = commands.get_entity(entity) {
                    let bounds = derived.wallprint.rects[0];
                    e.insert((
                        crate::objects::components::WallMountedPlacement { attachment },
                        crate::objects::components::WallMounted {
                            attachment,
                            width: spec.width,
                            height: spec.height,
                        },
                        derived.wallprint,
                        derived.interior_access_zone,
                        crate::objects::components::WallMountedBounds {
                            segment_key: bounds.segment_key,
                            offset_min: bounds.offset_min,
                            offset_max: bounds.offset_max,
                            height_min: bounds.height_min,
                            height_max: bounds.height_max,
                        },
                        crate::objects::components::ObjectPlacementComponent {
                            placement: effective_placement,
                        },
                    ));
                }
            } else {
                let new_wallprint =
                    derive_wallprint(attachment, spec.width, spec.height, occupancy_kind);

                // 3. Validate against other wallprints (ignoring self)
                {
                    for (existing_print, existing_id) in wallprints.iter() {
                        if existing_id.0 != stable_id
                            && wallprints_conflict(&new_wallprint, existing_print)
                        {
                            return DomainCommandResult::Rejected {
                                reason: DomainCommandRejectReason::WallMountedOverlap,
                            };
                        }
                    }
                }

                // 4. Update components
                if let Ok(mut e) = commands.get_entity(entity) {
                    let bounds = new_wallprint.rects[0];
                    e.insert((
                        crate::objects::components::WallMountedPlacement { attachment },
                        crate::objects::components::WallMounted {
                            attachment,
                            width: spec.width,
                            height: spec.height,
                        },
                        new_wallprint,
                        crate::objects::components::WallMountedBounds {
                            segment_key: bounds.segment_key,
                            offset_min: bounds.offset_min,
                            offset_max: bounds.offset_max,
                            height_min: bounds.height_min,
                            height_max: bounds.height_max,
                        },
                        crate::objects::components::ObjectPlacementComponent {
                            placement: effective_placement,
                        },
                    ));

                    if let Some(hit) = set.p3().iter().find(|s| s.key == attachment.segment_key) {
                        let v_pos = crate::store::wall_surface_world_pos(
                            hit,
                            attachment.offset_along_segment,
                        );
                        e.insert(crate::objects::components::WorldPos(v_pos));
                    }
                }
            }
        }
    }

    DomainCommandResult::Applied {
        events: vec![crate::store::events::DomainEvent::ObjectMoved { id: c.object_id }],
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
        Query<(
            Entity,
            &crate::objects::components::ObjectStableId,
            &crate::objects::components::ObjectPrototypeId,
        )>,
        Query<&crate::store::WallSurface>,
    )>,
    access_zones: &Query<(Entity, &InteriorAccessZone, Option<&ObjectStableId>)>,
    world_bounds: &crate::store::WorldBounds,
    store: &crate::store::area::StoreArea,
) -> DomainCommandResult {
    // 1. Get rotation variant and position
    let variant = {
        let q1 = set.p1();
        let Ok((rotatable, _, _, _, _)) = q1.get(entity) else {
            return DomainCommandResult::Rejected {
                reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
            };
        };
        if c.to_rotation >= rotatable.variants.len() {
            return DomainCommandResult::Rejected {
                reason: DomainCommandRejectReason::RotationInvalid,
            };
        }
        rotatable.variants[c.to_rotation].clone()
    };
    let stable_id = {
        let q2 = set.p2();
        let Ok((_, sid, _)) = q2.get(entity) else {
            return DomainCommandResult::Rejected {
                reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
            };
        };
        sid.0
    };
    let world_pos = {
        let q0 = set.p0();
        let Ok((_, pos, _, _)) = q0.get(entity) else {
            return DomainCommandResult::Rejected {
                reason: DomainCommandRejectReason::ObjectMissing { id: c.object_id },
            };
        };
        pos.0
    };

    // 2. Validate rotated placement
    let validation = {
        let footprints = set.p0();
        crate::placement::validate_placement(
            world_bounds,
            store,
            &footprints,
            access_zones,
            &variant.footprint,
            world_pos,
            crate::placement::PlacementValidationOptions {
                ignore_entity: Some(entity),
                ignore_stable_id: Some(stable_id),
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
