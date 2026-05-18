use bevy::prelude::*;
use std::collections::VecDeque;
use crate::npc::direction::NpcDirection;
use crate::npc::task::{NpcRole, NpcTask};
use crate::npc::archetype::NpcArchetypeId;

#[derive(Component)]
pub struct Npc;

#[derive(Component)]
pub struct NpcIdentity {
    pub stable_id: String,
    pub archetype_id: NpcArchetypeId,
    pub role: NpcRole,
}

#[derive(Component)]
pub struct Facing {
    pub direction: NpcDirection,
}

#[derive(Component)]
pub struct NpcLocomotion {
    pub speed: f32,
    pub snap_epsilon: f32,
    pub state: NpcLocomotionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcLocomotionState {
    Idle,
    Moving,
}

#[derive(Component, Default)]
pub struct PersonalTaskQueue {
    pub tasks: VecDeque<NpcTask>,
}

#[derive(Component, Default)]
pub struct AssignedTaskQueue {
    pub tasks: VecDeque<NpcTask>,
}

#[derive(Component)]
pub struct NpcPickable;

#[derive(Component)]
pub struct NpcPickBounds {
    pub offset: Vec2,
    pub size: Vec2,
}

#[derive(Component)]
pub struct NpcAnimationPlayer {
    pub current_action: crate::npc::archetype::NpcAnimActionId,
    pub current_direction: NpcDirection,
    pub frame_index: usize,
    pub timer: Timer,
}

#[derive(Component)]
pub struct NpcAnimationIntent {
    pub action: crate::npc::archetype::NpcAnimActionId,
    pub direction: Option<NpcDirection>,
}
