use bevy::prelude::*;
use std::collections::VecDeque;
use crate::objects::components::WorldPos;
use crate::npc::components::{NpcLocomotion, NpcLocomotionState, Facing};
use crate::npc::route::NpcRoute;
use crate::npc::direction::{npc_direction_from_delta, NpcDirectionMapping};

#[derive(Debug)]
pub struct NpcMovementStep {
    pub delta: Vec2,
    pub reached_end: bool,
}

pub fn advance_along_route(
    pos: &mut Vec2,
    waypoints: &mut VecDeque<Vec2>,
    speed: f32,
    dt: f32,
    snap_epsilon: f32,
) -> NpcMovementStep {
    let mut remaining_dist = speed * dt;
    let mut total_delta = Vec2::ZERO;
    let mut reached_end = false;

    while remaining_dist > 0.0 && !waypoints.is_empty() {
        let target = waypoints[0];
        let to_target = target - *pos;
        let dist_to_target = to_target.length();

        if dist_to_target <= remaining_dist || dist_to_target < snap_epsilon {
            // Reached waypoint
            let actual_move = to_target;
            *pos = target;
            total_delta += actual_move;
            remaining_dist -= dist_to_target;
            waypoints.pop_front();
            
            if waypoints.is_empty() {
                reached_end = true;
            }
        } else {
            // Move toward waypoint
            let move_dir = to_target.normalize();
            let actual_move = move_dir * remaining_dist;
            *pos += actual_move;
            total_delta += actual_move;
            remaining_dist = 0.0;
        }
    }

    NpcMovementStep {
        delta: total_delta,
        reached_end,
    }
}

pub fn advance_npc_locomotion(
    time: Res<Time>,
    mut query: Query<(
        &mut WorldPos,
        &mut NpcLocomotion,
        &mut NpcRoute,
        &mut Facing,
    )>,
) {
    let dt = time.delta_secs();
    let mapping = NpcDirectionMapping::default(); // Could be a resource

    for (mut world_pos, mut locomotion, mut route, mut facing) in query.iter_mut() {
        if route.waypoints.is_empty() {
            locomotion.state = NpcLocomotionState::Idle;
            continue;
        }

        locomotion.state = NpcLocomotionState::Moving;
        
        let step = advance_along_route(
            &mut world_pos.0,
            &mut route.waypoints,
            locomotion.speed,
            dt,
            locomotion.snap_epsilon,
        );

        if let Some(new_dir) = npc_direction_from_delta(step.delta, &mapping) {
            facing.direction = new_dir;
        }

        if step.reached_end {
            locomotion.state = NpcLocomotionState::Idle;
        }
    }
}
