use bevy::prelude::*;

use crate::input::{PointerContext, PointerTargets};
use crate::objects::prototypes::BuildPrototypeId;

#[derive(Resource, Debug, Default)]
pub struct ToolContext {
    pub hovered_object: Option<Entity>,
    pub hovered_widget: Option<Entity>,
    pub active: Option<ActiveToolAction>,
    pub pointer_world: Vec2,
    pub pointer_projected: Vec2,
    pub is_over_ui: bool,
}

impl ToolContext {
    pub fn sync_from_pointer(&mut self, pointer: &PointerContext, targets: &PointerTargets) {
        self.hovered_object = targets.world_object;
        self.hovered_widget = targets.world_widget;
        self.pointer_world = pointer.world_pos;
        self.pointer_projected = pointer.projected_pos;
        self.is_over_ui = pointer.over_ui;
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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
