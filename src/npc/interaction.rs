use bevy::prelude::*;
use crate::objects::components::{StableObjectId, ObjectPlacement, ObjectPlacementComponent};
use crate::objects::prototypes::NpcInteractionPointSpec;
use crate::npc::anchor::{NpcAnchor, NpcAnchorId, NpcAnchorOwner, AnchorLocalId, AnchorRoleFilter};
use crate::navigation::types::{NavigationSpace, NavigationPoint};

pub fn validate_interaction_point_spec(spec: &NpcInteractionPointSpec) -> Result<(), String> {
    if spec.id.is_empty() {
        return Err("Empty interaction point ID".to_string());
    }
    Ok(())
}

pub fn derive_object_interaction_anchors(
    object_id: StableObjectId,
    placement: &ObjectPlacementComponent,
    points: &[NpcInteractionPointSpec],
) -> Vec<NpcAnchor> {
    let mut anchors = Vec::new();
    let base_pos = match placement.placement {
        ObjectPlacement::Floor { world_pos, .. } => world_pos,
        ObjectPlacement::WallMounted { .. } => Vec2::ZERO, // Simplification
    };

    for point in points {
        // Simplification: local_offset applied directly to world pos
        let anchor_pos = base_pos + point.local_offset;

        anchors.push(NpcAnchor {
            id: NpcAnchorId {
                owner: NpcAnchorOwner::StoreObject(object_id),
                local_id: AnchorLocalId(Box::leak(point.id.clone().into_boxed_str())),
            },
            owner: NpcAnchorOwner::StoreObject(object_id),
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

    anchors
}
