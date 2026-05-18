use crate::objects::prototypes::BuildObjectId;
use crate::store::ChunkPurchaseValidation;
use crate::store::StoreChunkCoord;
use crate::tools::ToolMode;
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct ToolSessionState {
    pub active: Option<ActiveToolSession>,
}

#[derive(Resource, Debug, Default)]
pub struct ToolReturnState {
    pub previous: Option<ToolMode>,
}

#[derive(Debug, Clone)]
pub enum ActiveToolSession {
    Build(BuildToolSession),
    Move(MoveToolSession),
    Expansion(ExpansionToolSession),
}

impl ActiveToolSession {
    pub fn preview_entity(&self) -> Option<Entity> {
        match self {
            Self::Build(s) => Some(s.preview_entity()),
            Self::Move(s) => Some(s.preview_entity),
            Self::Expansion(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuildToolSession {
    Floor(FloorBuildSession),
    WallMounted(WallMountedBuildSession),
}

impl BuildToolSession {
    pub fn preview_entity(&self) -> Entity {
        match self {
            Self::Floor(session) => session.preview_entity,
            Self::WallMounted(session) => session.preview_entity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FloorBuildSession {
    pub prototype_id: BuildObjectId,
    pub preview_entity: Entity,
    pub rotation_index: usize,
    pub awaiting_fresh_click: bool,
}

#[derive(Debug, Clone)]
pub struct WallMountedBuildSession {
    pub prototype_id: BuildObjectId,
    pub preview_entity: Entity,
    pub current_attachment: Option<crate::objects::components::WallAttachmentPoint>,
    pub rotation_index: usize,
    pub awaiting_fresh_click: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MoveToolSession {
    pub source_entity: Entity,
    pub preview_entity: Entity,
    pub original_world_pos: Vec2,
    pub rotation_index: usize,
    pub awaiting_fresh_click: bool,
}

#[derive(Debug, Clone)]
pub struct ExpansionToolSession {
    pub hovered_coord: Option<StoreChunkCoord>,
    pub pending_confirm_coord: Option<StoreChunkCoord>,
    pub validation: Option<ChunkPurchaseValidation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSessionEndReason {
    #[allow(dead_code)]
    Committed,
    #[allow(dead_code)]
    Cancelled,
    Replaced,
    Returned,
}
