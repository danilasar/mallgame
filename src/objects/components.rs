use bevy::prelude::*;

use super::prototypes::BuildPrototypeId;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldPos(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ProjectedPos(pub Vec2);

/// Pixel offset from the sprite center to the contact point used for sorting and picking.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FootAnchor(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualOffset(pub Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortLayer {
    Floor,
    Decals,
    Objects,
    Characters,
    DragPreview,
    SelectionOverlay,
}

impl SortLayer {
    pub fn base_z(self) -> f32 {
        match self {
            SortLayer::Floor => -1000.0,
            SortLayer::Decals => -500.0,
            SortLayer::Objects => 0.0,
            SortLayer::Characters => 1000.0,
            SortLayer::DragPreview => 2000.0,
            SortLayer::SelectionOverlay => 3000.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SortBias(pub f32);

#[derive(Component, Debug, Clone)]
pub struct Footprint {
    pub local_polygon: Vec<Vec2>,
}

impl Footprint {
    pub fn rectangle(half_extents: Vec2) -> Self {
        Self {
            local_polygon: vec![
                Vec2::new(-half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, half_extents.y),
                Vec2::new(-half_extents.x, half_extents.y),
            ],
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlocksPlacement;

#[derive(Component, Debug, Clone, Copy)]
pub struct Interactive;

#[derive(Component, Debug, Clone, Copy)]
pub struct Movable;

#[derive(Component, Debug, Clone, Copy)]
pub struct Deletable;

#[derive(Component, Debug, Clone, Copy)]
pub struct BuildGhost;

#[derive(Component, Debug, Clone, Copy)]
pub struct GhostOf {
    pub prototype: BuildPrototypeId,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Selected;

#[derive(Component, Debug, Clone, Copy)]
pub struct HighlightIntent {
    pub kind: HighlightKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HighlightKind {
    Hover,
    Selected,
    MoveValid,
    MoveInvalid,
    DeleteDanger,
    BuildValid,
    BuildInvalid,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlaceableAssetId(pub &'static str);
