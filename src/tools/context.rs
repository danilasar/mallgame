use bevy::prelude::*;

use crate::input::{PointerContext, PointerTargets};
use crate::objects::prototypes::BuildObjectId;

#[derive(Resource, Debug, Clone)]
pub struct ToolContext {
    pub hovered_entity: Option<Entity>,
    pub world_pos: Vec2,
    pub pointer_over_ui: bool,

    #[allow(dead_code)]
    pub active: Option<ActiveToolAction>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            hovered_entity: None,
            world_pos: Vec2::ZERO,
            pointer_over_ui: false,
            active: None,
        }
    }
}

impl ToolContext {
    pub fn sync_from_pointer(&mut self, pointer: &PointerContext, targets: &PointerTargets) {
        self.world_pos = pointer.world_pos;
        self.pointer_over_ui = pointer.over_ui;
        self.hovered_entity = targets.world_object;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ActiveToolAction {
    Building {
        prototype: BuildObjectId,
        ghost: Entity,
    },
    Moving {
        entity: Entity,
        original_pos: Vec2,
    },
    PendingDelete {
        entity: Entity,
    },
}
