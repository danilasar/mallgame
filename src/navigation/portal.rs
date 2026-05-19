use bevy::prelude::*;
use crate::npc::job::NpcRole;
use crate::objects::components::{StableObjectId, WallAttachmentPoint};
use crate::npc::anchor::{NpcAnchorId, NpcAnchorOwner, AnchorLocalId, NpcAnchor, NpcAnchorKind, AnchorRoleFilter, AnchorReservationPolicy};
use crate::navigation::types::{NavigationSpace, NavigationPoint};

#[derive(Debug, Clone)]
pub struct NavigationPortalSpec {
    pub kind: NavigationPortalKind,
    pub interior_local_offset: Vec2,
    pub exterior_local_offset: Vec2,
    pub allowed_roles: Vec<NpcRole>,
    pub bidirectional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationPortalKind {
    Doorway,
    Entrance,
    ExitOnly,
    StaffOnlyDoor,
}

#[derive(Component)]
pub struct NavigationPortal {
    pub id: NavigationPortalId,
    pub owner: StableObjectId,
    pub kind: NavigationPortalKind,
    pub a: NpcAnchorId,
    pub b: NpcAnchorId,
    pub traversal_policy: PortalTraversalPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavigationPortalId(pub u64);

pub struct PortalTraversalPolicy {
    pub allowed_roles: Vec<NpcRole>,
    pub bidirectional: bool,
}

pub fn derive_portal_anchors(
    owner_id: StableObjectId,
    attachment: WallAttachmentPoint,
    spec: &NavigationPortalSpec,
    wall_surface: &crate::store::WallSurface,
) -> (NpcAnchor, NpcAnchor) {
    let base_world = crate::store::wall_surface_world_pos(wall_surface, attachment.offset_along_segment);
    let wall_normal = wall_surface.normal;
    
    // Simplification: interior is along normal, exterior is opposite normal?
    // Usually normal points "out" or "in"? 
    // In Stage 5B, normal was calculated as perp to wall direction.
    // Let's assume interior is +normal and exterior is -normal.
    
    let interior_pos = base_world + wall_normal * spec.interior_local_offset.y;
    let exterior_pos = base_world - wall_normal * spec.exterior_local_offset.y;

    let interior_anchor = NpcAnchor {
        id: NpcAnchorId {
            owner: NpcAnchorOwner::Doorway(owner_id),
            local_id: AnchorLocalId("portal.interior"),
        },
        owner: NpcAnchorOwner::Doorway(owner_id),
        kind: NpcAnchorKind::PortalInteriorSide,
        point: NavigationPoint {
            space: NavigationSpace::World,
            pos: interior_pos,
        },
        facing: None,
        allowed_roles: AnchorRoleFilter { allowed: spec.allowed_roles.clone() },
        reservation_policy: AnchorReservationPolicy::None,
    };

    let exterior_anchor = NpcAnchor {
        id: NpcAnchorId {
            owner: NpcAnchorOwner::Doorway(owner_id),
            local_id: AnchorLocalId("portal.exterior"),
        },
        owner: NpcAnchorOwner::Doorway(owner_id),
        kind: NpcAnchorKind::PortalExteriorSide,
        point: NavigationPoint {
            space: NavigationSpace::World,
            pos: exterior_pos,
        },
        facing: None,
        allowed_roles: AnchorRoleFilter { allowed: spec.allowed_roles.clone() },
        reservation_policy: AnchorReservationPolicy::None,
    };

    (interior_anchor, exterior_anchor)
}
