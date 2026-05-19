use bevy::prelude::*;
use crate::objects::components::{StableObjectId, ObjectPlacement, ObjectPlacementComponent};
use crate::navigation::types::{NavigationPoint, NavigationSpace};
use crate::npc::direction::NpcDirection;
use crate::npc::job::NpcRole;
use crate::store::events::DomainEvent;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NpcAnchorId {
    pub owner: NpcAnchorOwner,
    pub local_id: AnchorLocalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcAnchorOwner {
    StoreObject(StableObjectId),
    Doorway(StableObjectId),
    WorldMarker(Entity),
    Debug(Entity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnchorLocalId(pub &'static str);

pub struct NpcAnchor {
    pub id: NpcAnchorId,
    pub owner: NpcAnchorOwner,
    pub kind: NpcAnchorKind,
    pub point: NavigationPoint,
    pub facing: Option<NpcDirection>,
    pub allowed_roles: AnchorRoleFilter,
    pub reservation_policy: AnchorReservationPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcAnchorKind {
    Debug,
    MoveTarget,
    PortalInteriorSide,
    PortalExteriorSide,
    EntranceApproach,
    ExitApproach,
    BrowseProducts,
    CheckoutCustomer,
    CheckoutCashier,
    QueueSlot,
    Restock,
    StaffStation,
    Talk,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorReservationPolicy {
    None,
    SingleOccupant,
    QueueSlot,
    StaffOnly,
    CustomerOnly,
    RoleFiltered,
}

#[derive(Debug, Clone, Default)]
pub struct AnchorRoleFilter {
    pub allowed: Vec<NpcRole>,
}

#[derive(Resource, Default)]
pub struct NpcAnchorCache {
    pub by_id: HashMap<NpcAnchorId, NpcAnchor>,
    pub by_owner: HashMap<NpcAnchorOwner, Vec<NpcAnchorId>>,
}

impl NpcAnchorCache {
    pub fn insert(&mut self, anchor: NpcAnchor) {
        let id = anchor.id;
        let owner = anchor.owner;
        self.by_id.insert(id, anchor);
        self.by_owner.entry(owner).or_default().push(id);
    }

    pub fn remove_by_owner(&mut self, owner: NpcAnchorOwner) {
        if let Some(ids) = self.by_owner.remove(&owner) {
            for id in ids {
                self.by_id.remove(&id);
            }
        }
    }
}

pub enum NpcAnchorResolveError {
    OwnerMissing,
    AnchorMissing,
    RoleForbidden,
    AnchorNotReachable,
    UnsupportedTarget,
}

pub fn resolve_anchor_for_npc(
    npc_role: NpcRole,
    target: crate::npc::job::NpcMoveTarget,
    cache: &NpcAnchorCache,
) -> Result<&NpcAnchor, NpcAnchorResolveError> {
    let anchor_id = match target {
        crate::npc::job::NpcMoveTarget::Point(_) => {
            return Err(NpcAnchorResolveError::UnsupportedTarget);
        }
        crate::npc::job::NpcMoveTarget::InteractionPoint { object_id, point_id } => NpcAnchorId {
            owner: NpcAnchorOwner::StoreObject(object_id),
            local_id: crate::npc::anchor::AnchorLocalId(point_id.0),
        },
    };

    let anchor = cache
        .by_id
        .get(&anchor_id)
        .ok_or(NpcAnchorResolveError::AnchorMissing)?;

    if !anchor.allowed_roles.allowed.is_empty() && !anchor.allowed_roles.allowed.contains(&npc_role)
    {
        return Err(NpcAnchorResolveError::RoleForbidden);
    }

    Ok(anchor)
}

pub fn anchor_lifecycle_system(
    mut events: MessageReader<DomainEvent>,
    mut cache: ResMut<NpcAnchorCache>,
    catalog: Res<crate::objects::prototypes::ObjectCatalog>,
    objects: Query<(
        &crate::objects::components::ObjectStableId,
        &ObjectPlacementComponent,
        &crate::objects::components::ObjectPrototypeId,
        Option<&crate::objects::components::NpcInteractionPoints>,
    )>,
    wall_surfaces: Query<&crate::store::WallSurface>,
) {
    for event in events.read() {
        match event {
            DomainEvent::ObjectBuilt { id } | DomainEvent::ObjectMoved { id } => {
                cache.remove_by_owner(NpcAnchorOwner::StoreObject(*id));
                cache.remove_by_owner(NpcAnchorOwner::Doorway(*id));

                if let Some((_, placement, proto_id, interaction_points)) =
                    objects.iter().find(|(sid, _, _, _)| &sid.0 == id)
                {
                    if let Some(proto) = catalog.prototypes.get(&proto_id.0) {
                        derive_anchors_for_object(
                            *id,
                            placement,
                            proto,
                            interaction_points,
                            &mut cache,
                            &wall_surfaces,
                        );
                    }
                }
            }
            DomainEvent::ObjectDeleted { id } => {
                cache.remove_by_owner(NpcAnchorOwner::StoreObject(*id));
                cache.remove_by_owner(NpcAnchorOwner::Doorway(*id));
            }
            _ => {}
        }
    }
}

fn derive_anchors_for_object(
    id: StableObjectId,
    placement: &ObjectPlacementComponent,
    proto: &crate::objects::prototypes::ObjectPrototype,
    interaction_points: Option<&crate::objects::components::NpcInteractionPoints>,
    cache: &mut NpcAnchorCache,
    wall_surfaces: &Query<&crate::store::WallSurface>,
) {
    // 1. Check for NavigationPortal capability
    if let Some(portal_spec) = proto.capabilities.iter().find_map(|cap| {
        if let crate::objects::prototypes::ObjectCapabilitySpec::NavigationPortal(spec) = cap {
            Some(spec)
        } else {
            None
        }
    }) {
        if let ObjectPlacement::WallMounted { attachment } = placement.placement {
            if let Some(wall_surface) = wall_surfaces.iter().find(|s| s.key == attachment.segment_key)
            {
                let (interior, exterior) = crate::navigation::portal::derive_portal_anchors(
                    id,
                    attachment,
                    portal_spec,
                    wall_surface,
                );
                cache.insert(interior);
                cache.insert(exterior);
            }
        }
    }

    // 2. Interaction Points
    if let Some(points) = interaction_points {
        for point in &points.points {
            let base_pos = match placement.placement {
                ObjectPlacement::Floor { world_pos, .. } => world_pos,
                ObjectPlacement::WallMounted { attachment } => {
                    if let Some(wall_surface) =
                        wall_surfaces.iter().find(|s| s.key == attachment.segment_key)
                    {
                        crate::store::wall_surface_world_pos(
                            wall_surface,
                            attachment.offset_along_segment,
                        )
                    } else {
                        Vec2::ZERO
                    }
                }
            };

            // Simplification: local_offset is applied to world position for now.
            // For wall-mounted it should probably account for wall normal.
            let anchor_pos = base_pos + point.local_offset;

            cache.insert(NpcAnchor {
                id: NpcAnchorId {
                    owner: NpcAnchorOwner::StoreObject(id),
                    local_id: AnchorLocalId(Box::leak(point.id.clone().into_boxed_str())), // Leak for demo simplicity, better use stable string keys
                },
                owner: NpcAnchorOwner::StoreObject(id),
                kind: point.kind,
                point: NavigationPoint {
                    space: NavigationSpace::World,
                    pos: anchor_pos,
                },
                facing: point.facing,
                allowed_roles: AnchorRoleFilter {
                    allowed: point.allowed_roles.clone(),
                },
                reservation_policy: point.reservation_policy,
            });
        }
    }

    // 3. Base anchor (fallback or MoveTarget)
    let pos = match placement.placement {
        ObjectPlacement::Floor { world_pos, .. } => world_pos,
        ObjectPlacement::WallMounted { attachment } => {
            if let Some(wall_surface) = wall_surfaces.iter().find(|s| s.key == attachment.segment_key)
            {
                crate::store::wall_surface_world_pos(wall_surface, attachment.offset_along_segment)
            } else {
                Vec2::ZERO
            }
        }
    };

    cache.insert(NpcAnchor {
        id: NpcAnchorId {
            owner: NpcAnchorOwner::StoreObject(id),
            local_id: AnchorLocalId("base"),
        },
        owner: NpcAnchorOwner::StoreObject(id),
        kind: NpcAnchorKind::MoveTarget,
        point: NavigationPoint {
            space: NavigationSpace::World,
            pos,
        },
        facing: None,
        allowed_roles: AnchorRoleFilter::default(),
        reservation_policy: AnchorReservationPolicy::None,
    });
}
