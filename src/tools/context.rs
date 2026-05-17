use bevy::prelude::*;

use crate::input::PointerContext;
use crate::objects::prototypes::BuildPrototypeId;

#[derive(Resource, Debug, Default)]
pub struct ToolContext {
    pub hovered: Option<Entity>,
    pub active: Option<ActiveToolAction>,
    pub pointer_world: Vec2,
    pub pointer_projected: Vec2,
    pub is_over_ui: bool,
}

impl ToolContext {
    pub fn sync_from_pointer(&mut self, pointer: &PointerContext) {
        self.hovered = pointer.hovered_entity;
        self.pointer_world = pointer.world_pos;
        self.pointer_projected = pointer.projected_pos;
        self.is_over_ui = pointer.over_ui;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActiveToolAction {
    Moving {
        entity: Entity,
        original_world_pos: Vec2,
        current_world_pos: Vec2,
        valid: bool,
    },
    Building {
        prototype: BuildPrototypeId,
        ghost: Entity,
        current_world_pos: Vec2,
        valid: bool,
    },
    PendingDelete {
        entity: Entity,
    },
}
