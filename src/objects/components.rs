use bevy::prelude::*;

use super::prototypes::BuildPrototypeId;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldPos(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ProjectedPos(pub Vec2);

/// Pixel offset from the sprite center to the contact point used for sorting and picking.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FootAnchor(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualOffset(pub Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortLayer {
    Floor,
    StoreOverlay,
    Decals,
    Objects,
    Characters,
    DragPreview,
    SelectionOverlay,
}

impl SortLayer {
    pub const ALL: [SortLayer; 7] = [
        SortLayer::Floor,
        SortLayer::StoreOverlay,
        SortLayer::Decals,
        SortLayer::Objects,
        SortLayer::Characters,
        SortLayer::DragPreview,
        SortLayer::SelectionOverlay,
    ];

    pub fn base_z(self) -> f32 {
        match self {
            SortLayer::Floor => 100.0,
            SortLayer::StoreOverlay => 200.0,
            SortLayer::Decals => 300.0,
            SortLayer::Objects => 500.0,
            SortLayer::Characters => 600.0,
            SortLayer::DragPreview => 800.0,
            SortLayer::SelectionOverlay => 900.0,
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
#[allow(dead_code)]
pub struct GhostOf {
    pub prototype: BuildPrototypeId,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Selectable;

#[derive(Component, Debug, Clone, Copy)]
pub struct Inspectable;

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

impl HighlightKind {
    #[allow(dead_code)]
    pub fn priority(self) -> u8 {
        match self {
            HighlightKind::DeleteDanger => 100,
            HighlightKind::MoveInvalid | HighlightKind::BuildInvalid => 90,
            HighlightKind::MoveValid | HighlightKind::BuildValid => 70,
            HighlightKind::Selected => 50,
            HighlightKind::Hover => 10,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlaceableAssetId(pub &'static str);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionRole {
    WorldObject,
    WorldWidget,
    ToolPreview,
    Overlay,
    #[allow(dead_code)]
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOwner {
    ToolPreview,
    WorldWidget,
    ExpansionOverlay,
    FootprintOverlay,
    StoreOverlay,
    #[allow(dead_code)]
    SelectionHighlight,
    #[allow(dead_code)]
    DebugOverlay,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeOwned {
    pub owner: RuntimeOwner,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct StoreObject;
