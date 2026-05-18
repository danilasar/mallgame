use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerTargets};
use crate::tools::{
    ActiveToolSession, ExpansionToolSession, ReturnToPreviousToolRequested, ToolContext,
    ToolDescriptor, ToolInputGate, ToolMode, ToolRegistry, ToolSessionState, ToolSet,
};

pub struct ExpansionToolPlugin;

impl Plugin for ExpansionToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Expansion,
                action: InputAction::ToolCursor, // expansion usually contextual but can have hotkey
                label: "Expansion",
            });

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

fn start_expansion_session(mut session: ResMut<ToolSessionState>) {
    session.active = Some(ActiveToolSession::Expansion(ExpansionToolSession {
        hovered_coord: None,
        pending_confirm_coord: None,
        validation: None,
    }));
}

fn cleanup_expansion_session(mut commands: Commands, mut session: ResMut<ToolSessionState>) {
    crate::tools::cleanup_current_session(
        &mut commands,
        &mut session,
        crate::tools::ToolSessionEndReason::Replaced,
    );
}

pub fn expansion_tool_system(
    _commands: Commands,
    pointer: Res<PointerContext>,
    targets: Res<PointerTargets>,
    gate: Res<ToolInputGate>,
    _actions: Res<InputActionState>,
    _next_mode: ResMut<NextState<ToolMode>>,
    mut tool: ResMut<ToolContext>,
    mut session: ResMut<ToolSessionState>,
    world_bounds: Res<crate::store::WorldBounds>,
    store: Res<crate::store::StoreArea>,
    mut modal_requests: MessageWriter<crate::ui::ModalRequest>,
    mut return_to_previous: MessageWriter<ReturnToPreviousToolRequested>,
) {
    tool.sync_from_pointer(&pointer, &targets);

    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        return_to_previous.write(ReturnToPreviousToolRequested);
        return;
    }

    if let Some(ActiveToolSession::Expansion(expansion)) = session.active.as_mut() {
        let coord = store.world_to_chunk_coord(pointer.world_pos);
        expansion.hovered_coord = Some(coord);

        let validation = crate::store::expansion::validate_chunk_purchase(
            &world_bounds,
            &store,
            coord,
            crate::store::chunks::StoreChunkKind::Default,
        );
        let valid = validation.valid;
        expansion.validation = Some(validation);

        if gate.primary_world_click_released && valid {
            info!(
                "Expansion click detected at coord {:?}, opening modal",
                coord
            );
            expansion.pending_confirm_coord = Some(coord);
            modal_requests.write(crate::ui::ModalRequest::Open(
                crate::ui::ModalKind::ConfirmPurchaseChunk {
                    coord,
                    kind: crate::store::chunks::StoreChunkKind::Default,
                },
            ));
        } else if gate.primary_world_click_released {
            warn!(
                "Expansion click detected at coord {:?} but it is INVALID",
                coord
            );
        }
    }
}
