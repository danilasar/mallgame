use crate::objects::prototypes::BuildPrototypeId;
use crate::store::ChunkPurchaseValidation;
use crate::store::StoreChunkCoord;
use crate::tools::ToolMode;
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct ToolSessionState {
    pub active: Option<ActiveToolSession>,
}

#[derive(Debug)]
pub enum ActiveToolSession {
    Build(BuildToolSession),
    Move(MoveToolSession),
    Expansion(ExpansionToolSession),
}

#[derive(Debug)]
pub struct BuildToolSession {
    pub prototype_id: BuildPrototypeId,
    pub preview_entity: Entity,
    pub rotation_index: usize,
    pub awaiting_fresh_click: bool,
}

#[derive(Debug)]
pub struct MoveToolSession {
    pub source_entity: Entity,
    pub preview_entity: Entity,
    pub original_world_pos: Vec2,
    pub rotation_index: usize,
    pub awaiting_fresh_click: bool,
}

#[derive(Debug)]
pub struct ExpansionToolSession {
    pub hovered_coord: Option<StoreChunkCoord>,
    pub hovered_validation: Option<ChunkPurchaseValidation>,
    pub pending_modal_coord: Option<StoreChunkCoord>,
    pub awaiting_fresh_click: bool,
}

#[derive(Resource, Debug, Default)]
pub struct ToolReturnState {
    pub previous: Option<ToolMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolActivationKind {
    Replace,
    Temporary,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ActivateToolRequested {
    pub mode: ToolMode,
    pub kind: ToolActivationKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ReturnToPreviousToolRequested;

impl ActiveToolSession {
    pub fn preview_entity(&self) -> Option<Entity> {
        match self {
            ActiveToolSession::Build(s) => Some(s.preview_entity),
            ActiveToolSession::Move(s) => Some(s.preview_entity),
            ActiveToolSession::Expansion(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSessionEndReason {
    Cancelled,
    Replaced,
    Returned,
    Committed,
    #[allow(dead_code)]
    EntityDespawned,
    #[allow(dead_code)]
    UiSurfaceClosed,
}

#[allow(dead_code)]
pub fn despawn_runtime_owned(
    owner: crate::objects::components::RuntimeOwner,
    commands: &mut Commands,
    query: &Query<(Entity, &crate::objects::components::RuntimeOwned)>,
) {
    let mut count = 0;
    for (entity, owned) in query {
        if owned.owner == owner {
            commands.entity(entity).despawn();
            count += 1;
        }
    }
    if count > 0 {
        info!("Despawned {} entities owned by {:?}", count, owner);
    }
}
