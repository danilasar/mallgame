use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldPos(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PlacementState {
    Placed,
    Dragging,
    Preview,
    Blocked,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct InteractionState {
    pub selected: bool,
    pub hovered: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Draggable;

#[derive(Component, Debug, Clone, Copy)]
pub struct Selectable;

#[derive(Component, Debug, Clone, Copy)]
pub struct SelectionTint {
    pub normal: Color,
    pub selected: Color,
    pub dragging: Color,
    pub blocked: Color,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlaceableAssetId(pub &'static str);

/// Continuous collision shape around the object's base in simulation world space.
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionFootprint {
    pub half_extents: Vec2,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlocksPlacement;
