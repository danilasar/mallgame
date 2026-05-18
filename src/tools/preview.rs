use crate::objects::prototypes::BuildObjectId;
use bevy::prelude::*;

pub struct ToolPreviewPlugin;

impl Plugin for ToolPreviewPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ToolPreview;

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub enum ToolPreviewKind {
    Build { prototype_id: BuildObjectId },
    Move { source_entity: Entity },
    WallMounted { prototype_id: BuildObjectId },
}

#[derive(Component, Debug, Clone)]
pub struct PlacementPreview {
    pub validation: Option<Result<(), crate::store::PlacementInvalidReason>>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PreviewSource {
    pub preview_entity: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct NonInteractive;

#[derive(Component, Debug, Clone, Copy)]
pub struct WallMountedPreview;
