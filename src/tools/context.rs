use bevy::prelude::*;

use crate::input::{PointerContext, PointerTargets};

#[derive(Resource, Debug, Clone)]
pub struct ToolContext {
    pub hovered_entity: Option<Entity>,
    pub world_pos: Vec2,
    pub pointer_over_ui: bool,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            hovered_entity: None,
            world_pos: Vec2::ZERO,
            pointer_over_ui: false,
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
