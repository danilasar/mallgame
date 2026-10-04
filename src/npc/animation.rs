use bevy::prelude::*;
use crate::npc::components::*;
use crate::npc::archetype::{NpcCatalog, NpcAnimActionId, DirectionClipRef, ClipSpec, NpcArchetypeSpec};
use crate::npc::direction::NpcDirection;

pub fn update_animation_intent_from_locomotion(
    mut query: Query<(&NpcLocomotion, &Facing, &mut NpcAnimationIntent)>,
) {
    for (locomotion, facing, mut intent) in query.iter_mut() {
        match locomotion.state {
            NpcLocomotionState::Moving => {
                intent.action = NpcAnimActionId("base.walk".to_string());
                intent.direction = Some(facing.direction);
            }
            NpcLocomotionState::Idle => {
                intent.action = NpcAnimActionId("base.idle".to_string());
                intent.direction = Some(facing.direction);
            }
        }
    }
}

pub fn tick_npc_animation(
    time: Res<Time>,
    mut query: Query<&mut NpcAnimationPlayer>,
) {
    for mut player in query.iter_mut() {
        player.timer.tick(time.delta());
        if player.timer.just_finished() {
            player.frame_index += 1;
            // Looping and frame count would come from the resolved ClipSpec
            // For now, we just increment it. The actual sprite update will handle wrapping.
        }
    }
}

pub fn resolve_direction_clip<'a>(
    archetype: &'a NpcArchetypeSpec,
    action: &NpcAnimActionId,
    direction: NpcDirection,
) -> Option<&'a DirectionClipRef> {
    if let Some(directional_spec) = archetype.visuals.actions.get(action) {
        if let Some(clip_ref) = directional_spec.clips.get(&direction) {
            return Some(clip_ref);
        }
        // Fallback to default direction if specified
        if let Some(default_dir) = directional_spec.default_direction {
            if let Some(clip_ref) = directional_spec.clips.get(&default_dir) {
                return Some(clip_ref);
            }
        }
    }

    // Secondary fallback: fallback_action
    if action != &archetype.visuals.fallback_action {
        return resolve_direction_clip(archetype, &archetype.visuals.fallback_action, direction);
    }

    None
}

// System to sync Sprite with NpcAnimationPlayer + NpcAnimationIntent + Archetype
pub fn resolve_npc_animation_sprite(
    catalog: Res<NpcCatalog>,
    npc_query: Query<(&NpcIdentity, &NpcAnimationIntent)>,
    mut visual_query: Query<(&ChildOf, &mut NpcAnimationPlayer, &Sprite)>,
) {
    for (parent, mut player, _sprite) in visual_query.iter_mut() {
        let Ok((identity, intent)) = npc_query.get(parent.0) else {
            continue;
        };
        let Some(archetype) = catalog.archetypes.get(&identity.archetype_id) else {
            continue;
        };

        let action = &intent.action;
        let direction = intent.direction.unwrap_or(player.current_direction);

        if let Some(clip_ref) = resolve_direction_clip(archetype, action, direction) {
            match clip_ref {
                DirectionClipRef::Clip(ClipSpec::SingleSprite {
                    asset_id: _,
                    asset_path: _,
                }) => {
                    // Apply image to sprite
                }
                _ => {}
            }
        }

        player.current_action = action.clone();
        player.current_direction = direction;
    }
}

