use bevy::prelude::*;
use std::collections::HashMap;
use crate::npc::direction::NpcDirection;
use crate::npc::task::{NpcRole, NpcTaskProfileSpec};

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
    pub task_profile: NpcTaskProfileSpec,
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
