use bevy::prelude::*;

use crate::input::PointerContext;
use crate::store::{
    StoreArea, StoreChunkCoord, StoreChunkKind, WorldBounds, validate_chunk_purchase,
};
use crate::tools::{ToolContext, ToolInputGate, ToolMode, ToolSet};
use crate::ui::{ModalKind, ModalRequest};

#[derive(Resource, Debug, Default)]
pub struct ExpansionToolState {
    pub hovered_chunk: Option<StoreChunkCoord>,
    pub hovered_valid: bool,
}

pub struct ExpansionToolPlugin;

impl Plugin for ExpansionToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExpansionToolState>()
            .add_systems(OnExit(ToolMode::Expansion), clear_expansion_tool_state)
            .add_systems(
                Update,
                expansion_tool_system
                    .run_if(in_state(ToolMode::Expansion))
                    .in_set(ToolSet::ToolUpdate),
            );
    }
}

fn expansion_tool_system(
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    world: Res<WorldBounds>,
    store: Res<StoreArea>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    mut tool: ResMut<ToolContext>,
    mut state: ResMut<ExpansionToolState>,
    mut modal_requests: MessageWriter<ModalRequest>,
) {
    tool.sync_from_pointer(&pointer);
    if !gate.can_use_world() {
        return;
    }
    if gate.cancel_requested {
        state.hovered_chunk = None;
        state.hovered_valid = false;
        next_mode.set(ToolMode::Cursor);
        return;
    }

    let coord = store.world_to_chunk_coord(pointer.world_pos);
    let validation = validate_chunk_purchase(&world, &store, coord, StoreChunkKind::Default);
    let valid = validation.valid;
    if state.hovered_chunk != Some(coord) {
        info!(
            "Hovered expansion chunk {:?}, valid={}, reason={:?}",
            coord, valid, validation.reason
        );
    }
    state.hovered_chunk = Some(coord);
    state.hovered_valid = valid;

    if gate.primary_click_released && valid {
        modal_requests.write(ModalRequest::Open(ModalKind::ConfirmPurchaseChunk {
            coord,
            kind: StoreChunkKind::Default,
        }));
    }
}

fn clear_expansion_tool_state(mut state: ResMut<ExpansionToolState>) {
    state.hovered_chunk = None;
    state.hovered_valid = false;
}
