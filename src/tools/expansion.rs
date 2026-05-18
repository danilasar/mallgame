use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerTargets};
use crate::tools::{
    ActiveToolSession, ExpansionToolSession, ReturnToPreviousToolRequested, ToolContext,
    ToolDescriptor, ToolInputGate, ToolMode, ToolRegistry, ToolSessionState, ToolSet,
};
use bevy::ecs::system::SystemParam;

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

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct ExpansionToolParams<'w, 's> {
    _commands: Commands<'w, 's>,
    pointer: Res<'w, PointerContext>,
    targets: Res<'w, PointerTargets>,
    gate: Res<'w, ToolInputGate>,
    _actions: Res<'w, InputActionState>,
    _next_mode: ResMut<'w, NextState<ToolMode>>,
    tool: ResMut<'w, ToolContext>,
    session: ResMut<'w, ToolSessionState>,
    world_bounds: Res<'w, crate::store::WorldBounds>,
    store: Res<'w, crate::store::StoreArea>,
    modal_requests: MessageWriter<'w, crate::ui::ModalRequest>,
    return_to_previous: MessageWriter<'w, ReturnToPreviousToolRequested>,
}

pub fn expansion_tool_system(mut params: ExpansionToolParams) {
    params
        .tool
        .sync_from_pointer(&params.pointer, &params.targets);

    if !params.gate.can_use_world() {
        return;
    }

    if params.gate.cancel_requested {
        params
            .return_to_previous
            .write(ReturnToPreviousToolRequested);
        return;
    }

    if let Some(ActiveToolSession::Expansion(expansion)) = params.session.active.as_mut() {
        let coord = params.store.world_to_chunk_coord(params.pointer.world_pos);
        expansion.hovered_coord = Some(coord);

        let validation = crate::store::expansion::validate_chunk_purchase(
            &params.world_bounds,
            &params.store,
            coord,
            crate::store::chunks::StoreChunkKind::Default,
        );
        let valid = validation.valid;
        expansion.validation = Some(validation);

        if params.gate.primary_world_click_released && valid {
            info!(
                "Expansion click detected at coord {:?}, opening modal",
                coord
            );
            expansion.pending_confirm_coord = Some(coord);
            params.modal_requests.write(crate::ui::ModalRequest::Open(
                crate::ui::ModalKind::ConfirmPurchaseChunk {
                    coord,
                    kind: crate::store::chunks::StoreChunkKind::Default,
                },
            ));
        } else if params.gate.primary_world_click_released {
            warn!(
                "Expansion click detected at coord {:?} but it is INVALID",
                coord
            );
        }
    }
}
