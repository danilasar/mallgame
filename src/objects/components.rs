use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::prototypes::BuildObjectId;
use crate::store::WallSegmentKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableObjectId(pub u64);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectStableId(pub StableObjectId);

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPrototypeId(pub BuildObjectId);

#[derive(Resource, Debug, Clone, Copy)]
pub struct StableObjectIdAllocator {
    pub next: u64,
}

impl StableObjectIdAllocator {
    pub fn allocate(&mut self) -> StableObjectId {
        let id = StableObjectId(self.next);
        self.next += 1;
        id
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldPos(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ProjectedPos(pub Vec2);

/// Pixel offset from the sprite center to the contact point used for sorting and picking.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FootAnchor(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualOffset(pub Vec2);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallAttachmentPoint {
    pub segment_key: WallSegmentKey,
    pub offset_along_segment: f32,
    pub height_on_wall: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectPlacement {
    Floor {
        world_pos: Vec2,
        rotation_index: Option<usize>,
    },
    WallMounted {
        attachment: WallAttachmentPoint,
    },
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjectPlacementComponent {
    pub placement: ObjectPlacement,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallMounted {
    pub attachment: WallAttachmentPoint,
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallMountedBounds {
    pub segment_key: WallSegmentKey,
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallWindow {
    pub glass_alpha: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortLayer {
    Floor,
    ExteriorBack,
    StoreOverlay,
    WallFace,
    WallTopCap,
    Decals,
    Objects,
    Characters,
    DragPreview,
    SelectionOverlay,
}

impl SortLayer {
    pub const ALL: [SortLayer; 10] = [
        SortLayer::Floor,
        SortLayer::ExteriorBack,
        SortLayer::StoreOverlay,
        SortLayer::WallFace,
        SortLayer::WallTopCap,
        SortLayer::Decals,
        SortLayer::Objects,
        SortLayer::Characters,
        SortLayer::DragPreview,
        SortLayer::SelectionOverlay,
    ];

    pub fn base_z(self) -> f32 {
        match self {
            SortLayer::Floor => 100.0,
            SortLayer::ExteriorBack => 150.0,
            SortLayer::StoreOverlay => 200.0,
            SortLayer::WallFace => 240.0,
            SortLayer::WallTopCap => 245.0,
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

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct GhostOf {
    pub prototype: BuildObjectId,
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "future marker for debug picking")]
pub enum InteractionRole {
    WorldObject,
    WorldWidget,
    WallSurface,
    Exterior,
    ToolPreview,
    Overlay,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "future markers for selection/debug overlays")]
pub enum RuntimeOwner {
    ToolPreview,
    WorldWidget,
    BoundaryWall,
    Exterior,
    ExpansionOverlay,
    FootprintOverlay,
    StoreOverlay,
    SelectionHighlight,
    DebugOverlay,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeOwned {
    pub owner: RuntimeOwner,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct StoreObject;

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ExteriorObject;

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ExteriorStateful;

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ExteriorInspectable;

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ExteriorInteractive;

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ExteriorVisual;

// Stage 5A Capabilities

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ProductContainer {
    pub kind: super::prototypes::ProductContainerKind,
    pub capacity_class: super::prototypes::ContainerCapacityClass,
}

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct CheckoutPoint {
    pub kind: super::prototypes::CheckoutKind,
}

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Decor {
    pub kind: super::prototypes::DecorKind,
}

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct NpcInteractionPoints {
    pub points: Vec<NpcInteractionPoint>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct NpcInteractionPoint {
    pub local_pos: Vec2,
    pub facing: Vec2,
    pub kind: super::prototypes::NpcInteractionKind,
}
