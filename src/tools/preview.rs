use bevy::prelude::*;
use crate::objects::prototypes::BuildPrototypeId;
use crate::store::PlacementInvalidReason;

/// Marker for entities that are temporary tool previews.
#[derive(Component, Debug, Clone, Copy)]
pub struct ToolPreview;

/// Marker for entities that should not be interactive (not pickable).
#[derive(Component, Debug, Clone, Copy)]
pub struct NonInteractive;

#[derive(Component, Debug, Clone, Copy)]
pub enum ToolPreviewKind {
    Build {
        prototype_id: BuildPrototypeId,
    },
    Move {
        source_entity: Entity,
    },
}

#[derive(Component, Debug, Clone)]
pub struct PlacementPreview {
    pub validation: Option<Result<(), PlacementInvalidReason>>,
}

/// Attached to the original gameplay entity during a Move operation.
#[derive(Component, Debug, Clone, Copy)]
pub struct PreviewSource {
    pub preview_entity: Entity,
}

pub struct ToolPreviewPlugin;

impl Plugin for ToolPreviewPlugin {
    fn build(&self, _app: &mut App) {
        // Future systems for automated preview cleanup or visual updates can be added here
    }
}
