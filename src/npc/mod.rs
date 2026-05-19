pub mod direction;
pub mod archetype;
pub mod job;
pub mod components;
pub mod route;
pub mod locomotion;
pub mod animation;
pub mod picking;
pub mod presentation;
pub mod debug;
pub mod anchor;
pub mod interaction;

use bevy::prelude::*;
use crate::objects::components::*;
use crate::npc::components::*;
use crate::npc::archetype::NpcCatalog;
use crate::npc::job::{SpawnNpcRequested, DespawnNpcRequested, NpcRole};
use crate::npc::anchor::NpcAnchorCache;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<archetype::NpcCatalog>()
            .init_resource::<NpcAnchorCache>()
            .add_message::<job::SpawnNpcRequested>()
            .add_message::<job::PushNpcJobRequested>()
            .add_message::<job::DespawnNpcRequested>()
            .add_systems(Update, (
                handle_spawn_npc_requested,
                handle_despawn_npc_requested,
                job::handle_push_npc_job_requested,
                job::start_next_npc_job,
                anchor::anchor_lifecycle_system,
            ))
            .add_systems(Update, (
                locomotion::advance_npc_locomotion,
                animation::update_animation_intent_from_locomotion,
                animation::tick_npc_animation,
                animation::resolve_npc_animation_sprite,
                presentation::sync_npc_visual_transform,
            ));
    }
}

pub fn handle_spawn_npc_requested(
    mut commands: Commands,
    mut events: MessageReader<SpawnNpcRequested>,
    catalog: Res<NpcCatalog>,
) {
    for event in events.read() {
        let Some(archetype) = catalog.archetypes.get(&event.archetype_id) else {
            warn!("Failed to spawn NPC: Archetype {:?} not found", event.archetype_id);
            continue;
        };

        let role = event.role_override.unwrap_or(archetype.role);

        let root = commands.spawn((
            Npc,
            NpcIdentity {
                stable_id: format!("{}-{}", archetype.id.0, event.world_pos), // Simple stable ID for now
                archetype_id: archetype.id.clone(),
                role,
            },
            WorldPos(event.world_pos),
            Facing { direction: crate::npc::direction::NpcDirection::S }, // Default facing
            NpcLocomotion {
                speed: archetype.movement.speed,
                snap_epsilon: archetype.movement.snap_epsilon,
                state: NpcLocomotionState::Idle,
            },
            PersonalJobQueue::default(),
            InteractionRole::Npc,
            Interactive,
            SortLayer::Characters,
            FootAnchor(archetype.visuals.feet_anchor_px),
            VisualOffset(archetype.visuals.visual_offset_px),
            Transform::from_translation(Vec3::ZERO), // Sync system will update this
            InheritedVisibility::default(),
        )).id();

        if role != NpcRole::Customer {
            commands.entity(root).insert(AssignedJobQueue::default());
        }

        if archetype.picking.pickable {
            commands.entity(root).insert((
                NpcPickable,
                NpcPickBounds {
                    offset: archetype.picking.bounds.as_ref().map(|b| b.offset).unwrap_or(Vec2::ZERO),
                    size: archetype.picking.bounds.as_ref().map(|b| b.size).unwrap_or(Vec2::new(32.0, 64.0)),
                },
                Selectable,
                Inspectable,
            ));
        }

        // Child visual entity
        let visual = commands.spawn((
            NpcAnimationPlayer {
                current_action: archetype.visuals.fallback_action.clone(),
                current_direction: crate::npc::direction::NpcDirection::S,
                frame_index: 0,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            NpcAnimationIntent {
                action: archetype.visuals.fallback_action.clone(),
                direction: None,
            },
            Sprite::default(),
        )).id();
        
        commands.entity(root).add_child(visual);
    }
}

pub fn handle_despawn_npc_requested(
    mut commands: Commands,
    mut events: MessageReader<DespawnNpcRequested>,
) {
    for event in events.read() {
        if let Ok(mut entity) = commands.get_entity(event.npc) {
            entity.despawn(); // Use simple despawn if despawn_recursive is missing
        }
    }
}
