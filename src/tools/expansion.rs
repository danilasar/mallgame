use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext};
use crate::store::{
    StoreArea, StoreChunkKind, WorldBounds, validate_chunk_purchase,
};
use crate::tools::{
    ToolContext, ToolInputGate, ToolMode, ToolSet, ToolSessionState, ActiveToolSession,
    ExpansionToolSession, ReturnToPreviousToolRequested,
};
use crate::ui::{ModalKind, ModalRequest};

pub struct ExpansionToolPlugin;

impl Plugin for ExpansionToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ToolMode::Expansion), start_expansion_session)
            .add_systems(OnExit(ToolMode::Expansion), cleanup_expansion_session)
            .add_systems(
                Update,
                expansion_tool_system
                    .run_if(in_state(ToolMode::Expansion))
                    .in_set(ToolSet::ToolUpdate),
            );
    }
}

fn start_expansion_session(
    mut session: ResMut<ToolSessionState>,
) {
    session.active = Some(ActiveToolSession::Expansion(ExpansionToolSession {
        hovered_coord: None,
        hovered_validation: None,
        pending_modal_coord: None,
        awaiting_fresh_click: true,
    }));
}

fn cleanup_expansion_session(
    mut session: ResMut<ToolSessionState>,
) {
    if matches!(session.active, Some(ActiveToolSession::Expansion(_))) {
        session.active = None;
    }
}

fn expansion_tool_system(
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    actions: Res<InputActionState>,
    world: Res<WorldBounds>,
    store: Res<StoreArea>,
    mut tool: ResMut<ToolContext>,
    mut session: ResMut<ToolSessionState>,
    mut modal_requests: MessageWriter<ModalRequest>,
    mut return_request: MessageWriter<ReturnToPreviousToolRequested>,
) {
    tool.sync_from_pointer(&pointer);
    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        return_request.write(ReturnToPreviousToolRequested);
        return;
    }

    if let Some(ActiveToolSession::Expansion(expansion)) = session.active.as_mut() {
        // Reset freshness once button is fully released (and not in the release frame itself)
        if !actions.pressed(InputAction::PrimaryClick) && !actions.just_released(InputAction::PrimaryClick) {
            expansion.awaiting_fresh_click = false;
        }

        let coord = store.world_to_chunk_coord(pointer.world_pos);
        let validation = validate_chunk_purchase(&world, &store, coord, StoreChunkKind::Default);
        let valid = validation.valid;

        if expansion.hovered_coord != Some(coord) {
            info!(
                "Hovered expansion chunk {:?}, valid={}, reason={:?}",
                coord, valid, validation.reason
            );
        }

        expansion.hovered_coord = Some(coord);
        expansion.hovered_validation = Some(validation);

        if gate.primary_world_click_released && !expansion.awaiting_fresh_click && valid {
            expansion.pending_modal_coord = Some(coord);
            modal_requests.write(ModalRequest::Open(ModalKind::ConfirmPurchaseChunk {
                coord,
                kind: StoreChunkKind::Default,
            }));
        }
    }
}
