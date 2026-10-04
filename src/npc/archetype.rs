use bevy::prelude::*;
use std::collections::HashMap;
use crate::npc::direction::NpcDirection;
use crate::npc::job::{NpcRole, NpcJobProfileSpec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpcArchetypeId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpcAnimActionId(pub String);

pub struct NpcArchetypeSpec {
    pub id: NpcArchetypeId,
    pub role: NpcRole,
    pub movement: NpcMovementSpec,
    pub visuals: NpcVisualSpec,
    pub picking: NpcPickingSpec,
    pub job_profile: NpcJobProfileSpec,
}

pub struct NpcMovementSpec {
    pub speed: f32,
    pub snap_epsilon: f32,
}

pub struct NpcPickingSpec {
    pub pickable: bool,
    pub bounds: Option<NpcPickBoundsSpec>,
    pub pointer_occluder: bool,
}

pub struct NpcPickBoundsSpec {
    pub offset: Vec2,
    pub size: Vec2,
}

pub struct NpcVisualSpec {
    pub feet_anchor_px: Vec2,
    pub visual_offset_px: Vec2,
    pub sort_bias: f32,
    pub actions: HashMap<NpcAnimActionId, DirectionalAnimationSpec>,
    pub fallback_action: NpcAnimActionId,
}

pub struct DirectionalAnimationSpec {
    pub clips: HashMap<NpcDirection, DirectionClipRef>,
    pub default_direction: Option<NpcDirection>,
}

pub enum DirectionClipRef {
    Clip(ClipSpec),
    Mirrored {
        from: NpcDirection,
        flip_x: bool,
    },
    Fallback {
        action: NpcAnimActionId,
        direction: NpcDirection,
    },
}

pub enum ClipSpec {
    SingleSprite {
        asset_id: String,
        asset_path: String,
    },
    AtlasFrames {
        asset_id: String,
        asset_path: String,
        frames: Vec<usize>,
        fps: f32,
        looping: bool,
    },
}

#[derive(Resource, Default)]
pub struct NpcCatalog {
    pub archetypes: HashMap<NpcArchetypeId, NpcArchetypeSpec>,
}

#[derive(Debug)]
pub enum ArchetypeValidationError {
    EmptyId,
    InvalidSpeed,
    MissingBaseAction(String),
    MissingFallbackAction,
}

pub fn validate_npc_archetype(spec: &NpcArchetypeSpec) -> Result<(), ArchetypeValidationError> {
    if spec.id.0.is_empty() {
        return Err(ArchetypeValidationError::EmptyId);
    }
    if spec.movement.speed <= 0.0 {
        return Err(ArchetypeValidationError::InvalidSpeed);
    }
    if !spec.visuals.actions.contains_key(&NpcAnimActionId("base.idle".to_string())) {
        return Err(ArchetypeValidationError::MissingBaseAction("base.idle".to_string()));
    }
    if !spec.visuals.actions.contains_key(&NpcAnimActionId("base.walk".to_string())) {
        return Err(ArchetypeValidationError::MissingBaseAction("base.walk".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::job::NpcRole;
    use crate::npc::job::NpcJobProfileSpec;
    use std::collections::HashSet;

    fn test_archetype() -> NpcArchetypeSpec {
        let mut actions = HashMap::new();
        actions.insert(NpcAnimActionId("base.idle".to_string()), DirectionalAnimationSpec {
            clips: HashMap::new(),
            default_direction: None,
        });
        actions.insert(NpcAnimActionId("base.walk".to_string()), DirectionalAnimationSpec {
            clips: HashMap::new(),
            default_direction: None,
        });

        NpcArchetypeSpec {
            id: NpcArchetypeId("test".to_string()),
            role: NpcRole::Customer,
            movement: NpcMovementSpec {
                speed: 100.0,
                snap_epsilon: 1.0,
            },
            visuals: NpcVisualSpec {
                feet_anchor_px: Vec2::ZERO,
                visual_offset_px: Vec2::ZERO,
                sort_bias: 0.0,
                actions,
                fallback_action: NpcAnimActionId("base.idle".to_string()),
            },
            picking: NpcPickingSpec {
                pickable: true,
                bounds: None,
                pointer_occluder: false,
            },
            job_profile: NpcJobProfileSpec {
                role: NpcRole::Customer,
                allowed_personal_jobs: HashSet::new(),
                allowed_assigned_jobs: HashSet::new(),
                job_sources: HashSet::new(),
            },
        }
    }

    #[test]
    fn test_valid_archetype() {
        let spec = test_archetype();
        assert!(validate_npc_archetype(&spec).is_ok());
    }

    #[test]
    fn test_invalid_speed() {
        let mut spec = test_archetype();
        spec.movement.speed = 0.0;
        assert!(matches!(validate_npc_archetype(&spec), Err(ArchetypeValidationError::InvalidSpeed)));
    }
}
