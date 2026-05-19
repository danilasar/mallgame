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
pub struct NpcJobKindId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcJobQueueKind {
    Personal,
    Assigned,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcJobSource {
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
pub struct PushNpcJobRequested {
    pub npc: Entity,
    pub queue: NpcJobQueueKind,
    pub job: NpcJob,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DespawnNpcRequested {
    pub npc: Entity,
}

#[derive(Debug)]
pub struct NpcJob {
    pub kind: NpcJobKindId,
    pub payload: NpcJobPayload,
    pub requested_animation: Option<NpcAnimActionId>,
    pub source: NpcJobSource,
}

#[derive(Debug, Clone)]
pub enum NpcJobPayload {
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
    InteractionPoint {
        object_id: crate::objects::components::StableObjectId,
        point_id: crate::npc::anchor::AnchorLocalId,
    },
}

pub struct NpcJobProfileSpec {
    pub role: NpcRole,
    pub allowed_personal_jobs: HashSet<NpcJobKindId>,
    pub allowed_assigned_jobs: HashSet<NpcJobKindId>,
    pub job_sources: HashSet<NpcJobSource>,
}

#[derive(Debug)]
pub enum NpcJobValidationError {
    RoleMismatch,
    QueueMismatch,
    SourceNotAllowed,
    JobKindNotAllowed,
    PayloadMismatch,
}

pub fn validate_job_for_npc_role(
    role: NpcRole,
    profile: &NpcJobProfileSpec,
    queue: NpcJobQueueKind,
    job: &NpcJob,
) -> Result<(), NpcJobValidationError> {
    if role != profile.role {
        return Err(NpcJobValidationError::RoleMismatch);
    }

    if !profile.job_sources.contains(&job.source) {
        return Err(NpcJobValidationError::SourceNotAllowed);
    }

    match queue {
        NpcJobQueueKind::Personal => {
            if !profile.allowed_personal_jobs.contains(&job.kind) {
                return Err(NpcJobValidationError::JobKindNotAllowed);
            }
        }
        NpcJobQueueKind::Assigned => {
            if role == NpcRole::Customer {
                return Err(NpcJobValidationError::QueueMismatch);
            }
            if !profile.allowed_assigned_jobs.contains(&job.kind) {
                return Err(NpcJobValidationError::JobKindNotAllowed);
            }
        }
    }

    Ok(())
}

pub fn handle_push_npc_job_requested(
    mut events: MessageReader<PushNpcJobRequested>,
    catalog: Res<crate::npc::archetype::NpcCatalog>,
    mut query: Query<(
        &crate::npc::components::NpcIdentity,
        Option<&mut crate::npc::components::PersonalJobQueue>,
        Option<&mut crate::npc::components::AssignedJobQueue>,
    )>,
) {
    for event in events.read() {
        let Ok((identity, personal, assigned)) = query.get_mut(event.npc) else {
            continue;
        };

        let Some(archetype) = catalog.archetypes.get(&identity.archetype_id) else {
            continue;
        };

        if let Err(e) = validate_job_for_npc_role(
            identity.role,
            &archetype.job_profile,
            event.queue,
            &event.job,
        ) {
            warn!("Job validation failed: {:?}", e);
            continue;
        }

        match event.queue {
            NpcJobQueueKind::Personal => {
                if let Some(mut q) = personal {
                    q.jobs.push_back(event.job.clone());
                }
            }
            NpcJobQueueKind::Assigned => {
                if let Some(mut q) = assigned {
                    q.jobs.push_back(event.job.clone());
                }
            }
        }
    }
}

pub fn start_next_npc_job(
    mut query: Query<(
        Entity,
        &crate::npc::components::NpcIdentity,
        &crate::objects::components::WorldPos,
        &mut crate::npc::components::PersonalJobQueue,
        Option<&mut crate::npc::components::AssignedJobQueue>,
        &mut crate::npc::route::NpcRoute,
        &mut crate::npc::components::NpcAnimationIntent,
    )>,
    nav_graph: Res<crate::navigation::graph::NavigationGraph>,
    anchor_cache: Res<crate::npc::anchor::NpcAnchorCache>,
) {
    for (_entity, identity, world_pos, mut personal, assigned, mut route, mut anim_intent) in
        query.iter_mut()
    {
        if !route.waypoints.is_empty() {
            // Job in progress (locomotion is consuming route)
            continue;
        }

        // 1. Check Personal queue first (higher priority for Stage 6A)
        if let Some(job) = personal.jobs.pop_front() {
            execute_job(
                job,
                identity.role,
                world_pos.0,
                &mut route,
                &mut anim_intent,
                &nav_graph,
                &anchor_cache,
            );
            continue;
        }

        // 2. Check Assigned queue if staff
        if let Some(mut assigned_q) = assigned {
            if let Some(job) = assigned_q.jobs.pop_front() {
                execute_job(
                    job,
                    identity.role,
                    world_pos.0,
                    &mut route,
                    &mut anim_intent,
                    &nav_graph,
                    &anchor_cache,
                );
            }
        }
    }
}

fn execute_job(
    job: NpcJob,
    role: NpcRole,
    current_pos: Vec2,
    route: &mut crate::npc::route::NpcRoute,
    anim_intent: &mut crate::npc::components::NpcAnimationIntent,
    nav_graph: &crate::navigation::graph::NavigationGraph,
    anchor_cache: &crate::npc::anchor::NpcAnchorCache,
) {
    if let Some(action) = job.requested_animation {
        anim_intent.action = action;
    }

    match job.payload {
        NpcJobPayload::MoveTo { target } => match target {
            NpcMoveTarget::Point(target_pos) => {
                let request = crate::navigation::types::RoutePlanRequest {
                    start: crate::navigation::types::NavigationPoint {
                        space: crate::navigation::types::NavigationSpace::World,
                        pos: current_pos,
                    },
                    target: crate::navigation::types::NavigationPoint {
                        space: crate::navigation::types::NavigationSpace::World,
                        pos: target_pos,
                    },
                    agent: crate::navigation::types::AgentNavigationProfile {
                        radius: 16.0,
                        clearance: 4.0,
                        can_use_staff_only: role == NpcRole::Staff,
                    },
                };

                use crate::navigation::planner::RoutePlanner;
                if let Ok(nav_route) = nav_graph.plan_route(request) {
                    route.waypoints = nav_route.waypoints;
                } else {
                    // Fallback to Manhattan if graph planning fails
                    route.waypoints = crate::npc::route::build_manhattan_route(
                        current_pos,
                        target_pos,
                        crate::npc::route::RouteAxisOrder::XThenY,
                    );
                }
            }
            NpcMoveTarget::InteractionPoint { .. } => {
                if let Ok(anchor) = crate::npc::anchor::resolve_anchor_for_npc(role, target, anchor_cache) {
                    let request = crate::navigation::types::RoutePlanRequest {
                        start: crate::navigation::types::NavigationPoint {
                            space: crate::navigation::types::NavigationSpace::World,
                            pos: current_pos,
                        },
                        target: anchor.point,
                        agent: crate::navigation::types::AgentNavigationProfile {
                            radius: 16.0,
                            clearance: 4.0,
                            can_use_staff_only: role == NpcRole::Staff,
                        },
                    };
                    
                    use crate::navigation::planner::RoutePlanner;
                    if let Ok(nav_route) = nav_graph.plan_route(request) {
                        route.waypoints = nav_route.waypoints;
                        if let Some(facing) = anchor.facing {
                            anim_intent.direction = Some(facing);
                        }
                    }
                }
            }
        },
        NpcJobPayload::FaceDirection { direction } => {
            anim_intent.direction = Some(direction);
        }
        NpcJobPayload::Wait { .. } => {
            // Wait job implementation would need a timer component
        }
        _ => {}
    }
}

impl Clone for NpcJob {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            payload: match &self.payload {
                NpcJobPayload::MoveTo { target } => NpcJobPayload::MoveTo {
                    target: match target {
                        NpcMoveTarget::Point(p) => NpcMoveTarget::Point(*p),
                        NpcMoveTarget::InteractionPoint { object_id, point_id } => {
                            NpcMoveTarget::InteractionPoint {
                                object_id: *object_id,
                                point_id: *point_id,
                            }
                        }
                    },
                },
                NpcJobPayload::Wait { duration } => NpcJobPayload::Wait {
                    duration: *duration,
                },
                NpcJobPayload::FaceDirection { direction } => NpcJobPayload::FaceDirection {
                    direction: *direction,
                },
                NpcJobPayload::Unsupported => NpcJobPayload::Unsupported,
            },
            requested_animation: self.requested_animation.clone(),
            source: self.source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(role: NpcRole) -> NpcJobProfileSpec {
        let mut personal = HashSet::new();
        personal.insert(NpcJobKindId("base.move_to".to_string()));
        
        let mut assigned = HashSet::new();
        if role == NpcRole::Staff {
            assigned.insert(NpcJobKindId("staff.work".to_string()));
        }

        let mut sources = HashSet::new();
        sources.insert(NpcJobSource::Player);
        sources.insert(NpcJobSource::AiSelf);

        NpcJobProfileSpec {
            role,
            allowed_personal_jobs: personal,
            allowed_assigned_jobs: assigned,
            job_sources: sources,
        }
    }

    #[test]
    fn test_job_validation() {
        let customer_profile = create_test_profile(NpcRole::Customer);
        let staff_profile = create_test_profile(NpcRole::Staff);

        let move_job = NpcJob {
            kind: NpcJobKindId("base.move_to".to_string()),
            payload: NpcJobPayload::MoveTo { target: NpcMoveTarget::Point(Vec2::ZERO) },
            requested_animation: None,
            source: NpcJobSource::Player,
        };

        let staff_job = NpcJob {
            kind: NpcJobKindId("staff.work".to_string()),
            payload: NpcJobPayload::Unsupported,
            requested_animation: None,
            source: NpcJobSource::Player,
        };

        // Customer accepts personal move
        assert!(validate_job_for_npc_role(NpcRole::Customer, &customer_profile, NpcJobQueueKind::Personal, &move_job).is_ok());
        
        // Customer rejects assigned queue
        assert!(matches!(validate_job_for_npc_role(NpcRole::Customer, &customer_profile, NpcJobQueueKind::Assigned, &move_job), Err(NpcJobValidationError::QueueMismatch)));

        // Staff accepts assigned work
        assert!(validate_job_for_npc_role(NpcRole::Staff, &staff_profile, NpcJobQueueKind::Assigned, &staff_job).is_ok());

        // Customer rejects staff work
        assert!(matches!(validate_job_for_npc_role(NpcRole::Customer, &customer_profile, NpcJobQueueKind::Personal, &staff_job), Err(NpcJobValidationError::JobKindNotAllowed)));
    }
}
