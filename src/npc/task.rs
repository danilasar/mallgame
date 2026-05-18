use bevy::prelude::*;
use std::collections::HashSet;
use std::time::Duration;
use crate::npc::direction::NpcDirection;
use crate::npc::archetype::NpcAnimActionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcRole {
    Customer,
    Staff,
    Service,
    Visitor,
    DebugDummy,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NpcTaskKindId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcTaskQueueKind {
    Personal,
    Assigned,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcTaskSource {
    Player,
    Automation,
    AiSelf,
    Debug,
}

#[derive(Message, Debug, Clone)]
pub struct SpawnNpcRequested {
    pub archetype_id: crate::npc::archetype::NpcArchetypeId,
    pub world_pos: Vec2,
    pub role_override: Option<NpcRole>,
}

#[derive(Message, Debug, Clone)]
pub struct PushNpcTaskRequested {
    pub npc: Entity,
    pub queue: NpcTaskQueueKind,
    pub task: NpcTask,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DespawnNpcRequested {
    pub npc: Entity,
}

#[derive(Debug)]
pub struct NpcTask {
    pub kind: NpcTaskKindId,
    pub payload: NpcTaskPayload,
    pub requested_animation: Option<NpcAnimActionId>,
    pub source: NpcTaskSource,
}

#[derive(Debug, Clone)]
pub enum NpcTaskPayload {
    MoveTo {
        target: NpcMoveTarget,
    },
    Wait {
        duration: Duration,
    },
    FaceDirection {
        direction: NpcDirection,
    },
    // Future placeholders
    Unsupported,
}

#[derive(Debug, Clone, Copy)]
pub enum NpcMoveTarget {
    Point(Vec2),
}

pub struct NpcTaskProfileSpec {
    pub role: NpcRole,
    pub allowed_personal_tasks: HashSet<NpcTaskKindId>,
    pub allowed_assigned_tasks: HashSet<NpcTaskKindId>,
    pub task_sources: HashSet<NpcTaskSource>,
}

#[derive(Debug)]
pub enum NpcTaskValidationError {
    RoleMismatch,
    QueueMismatch,
    SourceNotAllowed,
    TaskKindNotAllowed,
    PayloadMismatch,
}

pub fn validate_task_for_npc_role(
    role: NpcRole,
    profile: &NpcTaskProfileSpec,
    queue: NpcTaskQueueKind,
    task: &NpcTask,
) -> Result<(), NpcTaskValidationError> {
    if role != profile.role {
        return Err(NpcTaskValidationError::RoleMismatch);
    }

    if !profile.task_sources.contains(&task.source) {
        return Err(NpcTaskValidationError::SourceNotAllowed);
    }

    match queue {
        NpcTaskQueueKind::Personal => {
            if !profile.allowed_personal_tasks.contains(&task.kind) {
                return Err(NpcTaskValidationError::TaskKindNotAllowed);
            }
        }
        NpcTaskQueueKind::Assigned => {
            if role == NpcRole::Customer {
                return Err(NpcTaskValidationError::QueueMismatch);
            }
            if !profile.allowed_assigned_tasks.contains(&task.kind) {
                return Err(NpcTaskValidationError::TaskKindNotAllowed);
            }
        }
    }

    Ok(())
}

pub fn handle_push_npc_task_requested(
    mut events: MessageReader<PushNpcTaskRequested>,
    catalog: Res<crate::npc::archetype::NpcCatalog>,
    mut query: Query<(
        &crate::npc::components::NpcIdentity,
        Option<&mut crate::npc::components::PersonalTaskQueue>,
        Option<&mut crate::npc::components::AssignedTaskQueue>,
    )>,
) {
    for event in events.read() {
        let Ok((identity, personal, assigned)) = query.get_mut(event.npc) else {
            continue;
        };

        let Some(archetype) = catalog.archetypes.get(&identity.archetype_id) else {
            continue;
        };

        if let Err(e) = validate_task_for_npc_role(
            identity.role,
            &archetype.task_profile,
            event.queue,
            &event.task,
        ) {
            warn!("Task validation failed: {:?}", e);
            continue;
        }

        match event.queue {
            NpcTaskQueueKind::Personal => {
                if let Some(mut q) = personal {
                    q.tasks.push_back(event.task.clone());
                }
            }
            NpcTaskQueueKind::Assigned => {
                if let Some(mut q) = assigned {
                    q.tasks.push_back(event.task.clone());
                }
            }
        }
    }
}

pub fn start_next_npc_task(
    mut query: Query<(
        Entity,
        &crate::objects::components::WorldPos,
        &mut crate::npc::components::PersonalTaskQueue,
        Option<&mut crate::npc::components::AssignedTaskQueue>,
        &mut crate::npc::route::NpcRoute,
        &mut crate::npc::components::NpcAnimationIntent,
    )>,
) {
    for (_entity, world_pos, mut personal, assigned, mut route, mut anim_intent) in query.iter_mut() {
        if !route.waypoints.is_empty() {
            // Task in progress (locomotion is consuming route)
            continue;
        }

        // 1. Check Personal queue first (higher priority for Stage 6A)
        if let Some(task) = personal.tasks.pop_front() {
            execute_task(task, world_pos.0, &mut route, &mut anim_intent);
            continue;
        }

        // 2. Check Assigned queue if staff
        if let Some(mut assigned_q) = assigned {
            if let Some(task) = assigned_q.tasks.pop_front() {
                execute_task(task, world_pos.0, &mut route, &mut anim_intent);
            }
        }
    }
}

fn execute_task(
    task: NpcTask,
    current_pos: Vec2,
    route: &mut crate::npc::route::NpcRoute,
    anim_intent: &mut crate::npc::components::NpcAnimationIntent,
) {
    if let Some(action) = task.requested_animation {
        anim_intent.action = action;
    }

    match task.payload {
        NpcTaskPayload::MoveTo { target } => {
            match target {
                NpcMoveTarget::Point(target_pos) => {
                    route.waypoints = crate::npc::route::build_manhattan_route(
                        current_pos,
                        target_pos,
                        crate::npc::route::RouteAxisOrder::XThenY,
                    );
                }
            }
        }
        NpcTaskPayload::FaceDirection { direction } => {
            anim_intent.direction = Some(direction);
        }
        NpcTaskPayload::Wait { .. } => {
            // Wait task implementation would need a timer component
        }
        _ => {}
    }
}

impl Clone for NpcTask {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            payload: match &self.payload {
                NpcTaskPayload::MoveTo { target } => NpcTaskPayload::MoveTo {
                    target: match target {
                        NpcMoveTarget::Point(p) => NpcMoveTarget::Point(*p),
                    },
                },
                NpcTaskPayload::Wait { duration } => NpcTaskPayload::Wait {
                    duration: *duration,
                },
                NpcTaskPayload::FaceDirection { direction } => NpcTaskPayload::FaceDirection {
                    direction: *direction,
                },
                NpcTaskPayload::Unsupported => NpcTaskPayload::Unsupported,
            },
            requested_animation: self.requested_animation.clone(),
            source: self.source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(role: NpcRole) -> NpcTaskProfileSpec {
        let mut personal = HashSet::new();
        personal.insert(NpcTaskKindId("base.move_to".to_string()));
        
        let mut assigned = HashSet::new();
        if role == NpcRole::Staff {
            assigned.insert(NpcTaskKindId("staff.work".to_string()));
        }

        let mut sources = HashSet::new();
        sources.insert(NpcTaskSource::Player);
        sources.insert(NpcTaskSource::AiSelf);

        NpcTaskProfileSpec {
            role,
            allowed_personal_tasks: personal,
            allowed_assigned_tasks: assigned,
            task_sources: sources,
        }
    }

    #[test]
    fn test_task_validation() {
        let customer_profile = create_test_profile(NpcRole::Customer);
        let staff_profile = create_test_profile(NpcRole::Staff);

        let move_task = NpcTask {
            kind: NpcTaskKindId("base.move_to".to_string()),
            payload: NpcTaskPayload::MoveTo { target: NpcMoveTarget::Point(Vec2::ZERO) },
            requested_animation: None,
            source: NpcTaskSource::Player,
        };

        let staff_task = NpcTask {
            kind: NpcTaskKindId("staff.work".to_string()),
            payload: NpcTaskPayload::Unsupported,
            requested_animation: None,
            source: NpcTaskSource::Player,
        };

        // Customer accepts personal move
        assert!(validate_task_for_npc_role(NpcRole::Customer, &customer_profile, NpcTaskQueueKind::Personal, &move_task).is_ok());
        
        // Customer rejects assigned queue
        assert!(matches!(validate_task_for_npc_role(NpcRole::Customer, &customer_profile, NpcTaskQueueKind::Assigned, &move_task), Err(NpcTaskValidationError::QueueMismatch)));

        // Staff accepts assigned work
        assert!(validate_task_for_npc_role(NpcRole::Staff, &staff_profile, NpcTaskQueueKind::Assigned, &staff_task).is_ok());

        // Customer rejects staff work
        assert!(matches!(validate_task_for_npc_role(NpcRole::Customer, &customer_profile, NpcTaskQueueKind::Personal, &staff_task), Err(NpcTaskValidationError::TaskKindNotAllowed)));
    }
}
