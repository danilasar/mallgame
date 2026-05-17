use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext};
use crate::objects::components::{Movable, Selected, StoreObject, WorldPos, FootAnchor, Footprint, VisualOffset, SortLayer, SortBias, ProjectedPos};
use crate::tools::{
    ActiveToolSession, MoveObjectCommitted, StartMoveObjectRequested, ToolContext, ToolDescriptor,
    ToolInputGate, ToolMode, ToolRegistry, ToolSet, ToolSessionState, MoveToolSession, ToolPreview, ToolPreviewKind, NonInteractive, PreviewSource, PlacementPreview,
};

pub struct MoveToolPlugin;

impl Plugin for MoveToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Move,
                action: InputAction::ToolMove,
                label: "Move",
            });

        app.add_systems(
            Update,
            (
                apply_start_move_object_requests,
                move_tool_system,
            )
                .chain()
                .run_if(in_state(ToolMode::Move))
                .in_set(ToolSet::ToolUpdate),
        )
        .add_systems(OnExit(ToolMode::Move), cleanup_move_session);
    }
}

pub fn apply_start_move_object_requests(
    mut commands: Commands,
    mut requests: MessageReader<StartMoveObjectRequested>,
    movable: Query<(&WorldPos, &FootAnchor, &Footprint, &VisualOffset, &Sprite, &SortBias, Option<&crate::objects::rotation::Rotatable>), (With<Movable>, With<StoreObject>)>,
    selected: Query<Entity, With<Selected>>,
    mut session: ResMut<ToolSessionState>,
) {
    for request in requests.read() {
        if let Ok((world_pos, foot_anchor, footprint, visual_offset, sprite, sort_bias, rotatable)) = movable.get(request.entity) {
            // Cleanup existing session if any
            crate::tools::cleanup_current_session(&mut commands, &mut session, crate::tools::ToolSessionEndReason::Replaced);

            for selected_entity in &selected {
                commands.entity(selected_entity).remove::<Selected>();
            }
            commands.entity(request.entity).insert(Selected);

            let rotation_index = rotatable.map_or(0, |r| r.current);

            // Spawn preview entity
            let preview_entity = commands.spawn((
                Sprite {
                    image: sprite.image.clone(),
                    custom_size: sprite.custom_size,
                    color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                    ..default()
                },
                *world_pos,
                ProjectedPos::default(),
                *foot_anchor,
                *visual_offset,
                SortLayer::DragPreview,
                *sort_bias,
                footprint.clone(),
                ToolPreview,
                ToolPreviewKind::Move { source_entity: request.entity },
                PlacementPreview { validation: None },
                NonInteractive,
                Name::new(format!("MovePreview of {:?}", request.entity)),
            )).id();

            commands.entity(request.entity).insert(PreviewSource { preview_entity });

            session.active = Some(ActiveToolSession::Move(MoveToolSession {
                source_entity: request.entity,
                preview_entity,
                original_world_pos: world_pos.0,
                rotation_index, 
                awaiting_fresh_click: true,
            }));
        }
    }
}

pub fn move_tool_system(
    mut commands: Commands,
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    actions: Res<InputActionState>,
    movable: Query<Entity, (With<Movable>, With<StoreObject>)>,
    mut ghost_positions: Query<(&mut WorldPos, &PlacementPreview), (With<ToolPreview>, Without<StoreObject>)>,
    mut session: ResMut<ToolSessionState>,
    mut committed: MessageWriter<MoveObjectCommitted>,
    mut requests: MessageWriter<StartMoveObjectRequested>,
    mut tool: ResMut<ToolContext>,
) {
    tool.sync_from_pointer(&pointer);
    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        crate::tools::cleanup_current_session(
            &mut commands,
            &mut session,
            crate::tools::ToolSessionEndReason::Cancelled,
        );
        return;
    }

    if let Some(ActiveToolSession::Move(move_session)) = session.active.as_mut() {
        // Reset freshness once button is fully released (and not in the release frame itself)
        if !actions.pressed(InputAction::PrimaryClick) && !actions.just_released(InputAction::PrimaryClick) {
            move_session.awaiting_fresh_click = false;
        }

        let source_entity = move_session.source_entity;
        let preview_entity = move_session.preview_entity;

        if let Ok((mut world_pos, preview)) = ghost_positions.get_mut(preview_entity) {
            world_pos.0 = pointer.world_pos;

            if gate.primary_world_click_released && !move_session.awaiting_fresh_click {
                let is_valid = preview.validation.as_ref().map_or(false, |r| r.is_ok());
                if is_valid {
                    committed.write(MoveObjectCommitted {
                        entity: source_entity,
                        old_pos: move_session.original_world_pos,
                        new_pos: world_pos.0,
                        rotation: move_session.rotation_index,
                    });
                }
                crate::tools::cleanup_current_session(
                    &mut commands,
                    &mut session,
                    if is_valid {
                        crate::tools::ToolSessionEndReason::Committed
                    } else {
                        crate::tools::ToolSessionEndReason::Cancelled
                    },
                );
            }
        }
        return;
    }

    // Start move via click
    if gate.primary_world_press_started {
        if let Some(entity) = tool.hovered.filter(|entity| movable.get(*entity).is_ok()) {
            requests.write(StartMoveObjectRequested { entity });
        }
    }
}

pub fn cleanup_move_session(
    mut commands: Commands,
    mut session: ResMut<ToolSessionState>,
) {
    crate::tools::cleanup_current_session(
        &mut commands,
        &mut session,
        crate::tools::ToolSessionEndReason::Replaced,
    );
}
