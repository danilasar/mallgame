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

#[derive(Component, Debug, Clone, Copy)]
#[expect(
    dead_code,
    reason = "floor placement authority for Stage 5B.3.1 migration"
)]
pub struct FloorPlacement {
    pub world_pos: WorldPos,
    pub rotation_index: Option<usize>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallMountedPlacement {
    pub attachment: WallAttachmentPoint,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallMounted {
    pub attachment: WallAttachmentPoint,
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Wallprint {
    pub rects: Vec<WallprintRect>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallprintRect {
    pub segment_key: WallSegmentKey,
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub occupancy_kind: WallOccupancyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "future wall occupancy policies")]
pub enum WallOccupancyKind {
    Solid,
    Opening,
    DecorativeOverlay,
    ServiceMount,
    Debug,
}

pub fn derive_wallprint_rect(
    attachment: WallAttachmentPoint,
    width: f32,
    height: f32,
    occupancy_kind: WallOccupancyKind,
) -> WallprintRect {
    let half_width = width * 0.5;
    WallprintRect {
        segment_key: attachment.segment_key,
        offset_min: attachment.offset_along_segment - half_width,
        offset_max: attachment.offset_along_segment + half_width,
        height_min: attachment.height_on_wall,
        height_max: attachment.height_on_wall + height,
        occupancy_kind,
    }
}

pub fn derive_wallprint(
    attachment: WallAttachmentPoint,
    width: f32,
    height: f32,
    occupancy_kind: WallOccupancyKind,
) -> Wallprint {
    Wallprint {
        rects: vec![derive_wallprint_rect(
            attachment,
            width,
            height,
            occupancy_kind,
        )],
    }
}

pub fn wallprint_rects_conflict(a: &WallprintRect, b: &WallprintRect) -> bool {
    if a.occupancy_kind == WallOccupancyKind::Debug || b.occupancy_kind == WallOccupancyKind::Debug
    {
        return false;
    }
    if a.segment_key != b.segment_key {
        return false;
    }

    let horizontal_overlap = a.offset_min < b.offset_max && a.offset_max > b.offset_min;
    let vertical_overlap = a.height_min < b.height_max && a.height_max > b.height_min;
    horizontal_overlap && vertical_overlap
}

pub fn wallprints_conflict(a: &Wallprint, b: &Wallprint) -> bool {
    a.rects.iter().any(|a_rect| {
        b.rects
            .iter()
            .any(|b_rect| wallprint_rects_conflict(a_rect, b_rect))
    })
}

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct InteriorAccessZone {
    pub polygon: Vec<Vec2>,
    pub reason: AccessZoneReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[allow(
    clippy::enum_variant_names,
    reason = "domain names intentionally share Access suffix"
)]
pub enum AccessZoneReason {
    DoorAccess,
    CheckoutAccess,
    ContainerAccess,
    ServiceAccess,
}

#[derive(Component, Debug, Clone, Copy)]
#[expect(dead_code, reason = "door gameplay systems will consume doorway kind")]
pub struct Doorway {
    pub kind: DoorwayKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorwayKind {
    CustomerEntrance,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DoorMovable;

/// Derived runtime component on a wall-mounted object that carries `WallOpeningSpec`.
/// Stores the resolved wall-local opening rectangle.
/// Not saved — rederived from prototype + `ObjectPlacement::WallMounted` at spawn/move/load.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WallOpeningComponent {
    pub segment_key: WallSegmentKey,
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub glass_color: Option<Color>,
    #[allow(dead_code)]
    pub frame_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct DerivedDoorPlacement {
    pub wallprint: Wallprint,
    pub interior_access_zone: InteriorAccessZone,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DoorAccessZonePreview;

#[derive(Component, Debug, Clone)]
pub struct AccessZonePreviewShape {
    pub polygon: Vec<Vec2>,
}

/// Runtime wall-rect cache used by current selection/inspection paths.
///
/// `Wallprint` is the authoritative wall occupancy geometry. This component is
/// kept as a narrow compatibility cache while presentation code is migrated to
/// wallprint-driven selection/highlight strategies.
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

/// Floor occupancy geometry. Kept as an alias while the codebase migrates from
/// the older `Footprint` name.
pub type FloorFootprint = Footprint;

/// Floor placement blocker. The historical name is kept for compatibility, but
/// it must only be used by floor validation.
#[derive(Component, Debug, Clone, Copy)]
pub struct BlocksPlacement;

#[derive(Component, Debug, Clone, Copy)]
pub struct Interactive;

#[derive(Component, Debug, Clone, Copy)]
pub struct Movable;

#[derive(Component, Debug, Clone, Copy)]
pub struct WallMovable;

#[derive(Component, Debug, Clone, Copy)]
pub struct WallMovePreview;

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
    Npc,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{StoreBoundarySide, StoreChunkCoord};

    fn test_segment_key() -> WallSegmentKey {
        WallSegmentKey {
            chunk: StoreChunkCoord { x: -1, y: -1 },
            side: StoreBoundarySide::Top,
        }
    }

    #[test]
    fn wallprint_conflict_requires_same_segment_and_overlapping_rects() {
        let key = test_segment_key();
        let a = derive_wallprint(
            WallAttachmentPoint {
                segment_key: key,
                offset_along_segment: 50.0,
                height_on_wall: 20.0,
            },
            20.0,
            10.0,
            WallOccupancyKind::Solid,
        );
        let overlapping = derive_wallprint(
            WallAttachmentPoint {
                segment_key: key,
                offset_along_segment: 55.0,
                height_on_wall: 24.0,
            },
            20.0,
            10.0,
            WallOccupancyKind::Opening,
        );
        let separated = derive_wallprint(
            WallAttachmentPoint {
                segment_key: key,
                offset_along_segment: 90.0,
                height_on_wall: 20.0,
            },
            20.0,
            10.0,
            WallOccupancyKind::Solid,
        );

        assert!(wallprints_conflict(&a, &overlapping));
        assert!(!wallprints_conflict(&a, &separated));
    }

    #[test]
    fn debug_wallprint_rects_do_not_block() {
        let key = test_segment_key();
        let solid = derive_wallprint(
            WallAttachmentPoint {
                segment_key: key,
                offset_along_segment: 50.0,
                height_on_wall: 20.0,
            },
            20.0,
            10.0,
            WallOccupancyKind::Solid,
        );
        let debug = derive_wallprint(
            WallAttachmentPoint {
                segment_key: key,
                offset_along_segment: 50.0,
                height_on_wall: 20.0,
            },
            20.0,
            10.0,
            WallOccupancyKind::Debug,
        );

        assert!(!wallprints_conflict(&solid, &debug));
    }
}

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
