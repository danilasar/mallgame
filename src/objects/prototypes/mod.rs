use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use super::components::*;
use super::rotation::{Rotatable, RotationVariant};
use crate::tools::{
    NonInteractive, PlacementPreview, ToolPreview, ToolPreviewKind, WallMountedPreview,
};

/// Stable identifier for an object prototype.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BuildObjectId(pub String);

impl BuildObjectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for BuildObjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ObjectCatalog {
    pub prototypes: HashMap<BuildObjectId, ObjectPrototype>,
}

#[derive(Debug, Clone)]
pub struct ObjectPrototype {
    pub id: BuildObjectId,
    pub display: ObjectDisplaySpec,
    pub catalog: ObjectCatalogSpec,
    pub placement: PlacementSpec,
    pub visuals: VisualSpec,
    pub rotation: RotationSpec,
    pub capabilities: Vec<ObjectCapabilitySpec>,
    #[allow(dead_code)]
    pub initial_state: ObjectInitialStateSpec,
}

#[derive(Debug, Clone)]
pub struct ObjectDisplaySpec {
    pub display_name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub icon: Option<String>, // Path to icon asset
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectCategory {
    Fixture,
    Service,
    Decor,
    #[allow(dead_code)]
    Store,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuildRibbonTab {
    Fixtures,
    Service,
    Decor,
    Walls,
    Store,
}

impl BuildRibbonTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fixtures => "Fixtures",
            Self::Service => "Service",
            Self::Decor => "Decor",
            Self::Walls => "Walls",
            Self::Store => "Store",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuildRibbonGroup {
    Shelves,
    #[allow(dead_code)]
    RacksHangers,
    #[allow(dead_code)]
    Fridges,
    Checkout,
    Decor,
    Walls,
    #[allow(dead_code)]
    Expansion,
}

impl BuildRibbonGroup {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Shelves => "Shelves",
            Self::RacksHangers => "Racks / Hangers",
            Self::Fridges => "Fridges",
            Self::Checkout => "Checkout",
            Self::Decor => "Decor",
            Self::Walls => "Walls",
            Self::Expansion => "Expansion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogAvailability {
    Available,
    #[allow(dead_code)]
    HiddenDevOnly,
    #[allow(dead_code)]
    PlaceholderDisabled,
}

#[derive(Debug, Clone)]
pub struct ObjectCatalogSpec {
    #[allow(dead_code)]
    pub category: ObjectCategory,
    pub ribbon_tab: BuildRibbonTab,
    pub ribbon_group: BuildRibbonGroup,
    pub sort_order: i32,
    pub availability: CatalogAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementKind {
    Floor,
    WallMounted,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlacementSpec {
    pub kind: PlacementKind,
    pub footprint_half_extents: Vec2,
    pub placement_blocker: bool,
    pub navigation_blocker: bool,
}

#[derive(Debug, Clone)]
pub struct VisualSpec {
    pub asset_path: String,
    pub asset_id: String,
    pub sprite_size: Vec2,
    pub foot_anchor: Vec2,
    pub sort_bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationKind {
    None,
    TwoVariants, // 0 and 90 degrees
    #[allow(dead_code)]
    FourVariants,
}

#[derive(Debug, Clone)]
pub struct RotationSpec {
    pub kind: RotationKind,
    pub rotated_asset_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ObjectCapabilitySpec {
    ProductContainer(ProductContainerSpec),
    CheckoutPoint(CheckoutPointSpec),
    Decor(DecorSpec),
    NpcInteractionPoints(NpcInteractionPointsSpec),
    WallMounted(WallMountedSpec),
    Window(WindowSpec),
    Doorway(DoorwaySpec),
    DoorMovable,
    WallOpening(WallOpeningSpec),
}

/// Prototype-level opening spec. Determines the visible cutout in the wall surface.
/// Explicit opt-in: only prototypes with this capability create wall openings.
#[derive(Debug, Clone)]
pub struct WallOpeningSpec {
    pub shape: WallOpeningShapeSpec,
    /// Tint of the translucent glass quad drawn over the opening. `None` = no glass.
    pub glass_color: Option<Color>,
    /// Tint of the opaque frame quad drawn over the glass. `None` = no frame.
    #[allow(dead_code)]
    pub frame_color: Option<Color>,
}

/// Shape of the wall opening in prototype definition.
#[derive(Debug, Clone)]
pub enum WallOpeningShapeSpec {
    Rect {
        width: f32,
        height: f32,
        anchor: WallOpeningAnchor,
    },
    /// Reserved for a future polygon backend.
    /// `validate_object_catalog` rejects this variant in Stage 5B.6.
    #[allow(dead_code)]
    Polygon {
        vertices: Vec<Vec2>,
        anchor: WallOpeningAnchor,
    },
}

/// Controls how `height_on_wall` maps to the opening's vertical bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallOpeningAnchor {
    /// Opening is centred at `attachment.height_on_wall`.
    Center,
    /// Bottom of the opening sits at `attachment.height_on_wall` (use for floor-level doors).
    BottomCenter,
}

#[derive(Debug, Clone)]
pub struct DoorwaySpec {
    pub access_width: f32,
    pub access_depth: f32,
    pub base_height_on_wall: f32,
    pub kind: crate::objects::components::DoorwayKind,
}

#[derive(Debug, Clone)]
pub struct WallMountedSpec {
    pub width: f32,
    pub height: f32,
    pub allowed_sides: Vec<crate::store::StoreBoundarySide>,
    pub default_height_on_wall: f32,
    pub movable: bool,
}

#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub width: f32,
    pub height: f32,
    pub glass_alpha: f32,
}

#[derive(Debug, Clone)]
pub struct ProductContainerSpec {
    pub container_kind: ProductContainerKind,
    pub capacity_class: ContainerCapacityClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductContainerKind {
    Shelf,
    Rack,
    Fridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerCapacityClass {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone)]
pub struct CheckoutPointSpec {
    pub checkout_kind: CheckoutKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckoutKind {
    BasicRegister,
}

#[derive(Debug, Clone)]
pub struct DecorSpec {
    pub decor_kind: DecorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecorKind {
    Plant,
    Sign,
    Misc,
}

#[derive(Debug, Clone)]
pub struct NpcInteractionPointsSpec {
    pub points: Vec<NpcInteractionPointSpec>,
}

#[derive(Debug, Clone)]
pub struct NpcInteractionPointSpec {
    pub local_pos: Vec2,
    pub facing: Vec2, // Direction vector
    pub kind: NpcInteractionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcInteractionKind {
    BrowseProducts,
    Checkout,
}

#[derive(Debug, Clone)]
pub enum ObjectInitialStateSpec {
    None,
}

#[derive(Resource, Debug, Default)]
pub struct BuildSelectionState {
    pub selected_prototype_id: Option<BuildObjectId>,
}

#[derive(Message, Debug, Clone)]
pub struct SelectBuildPrototypeRequested {
    pub prototype_id: BuildObjectId,
}

pub struct SpawnStoreObjectParams {
    pub stable_id: StableObjectId,
    pub prototype_id: BuildObjectId,
    pub placement: ObjectPlacement,
    pub derived_door: Option<DerivedDoorPlacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnObjectError {
    UnknownPrototype(BuildObjectId),
}

impl fmt::Display for SpawnObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnObjectError::UnknownPrototype(id) => {
                write!(f, "Unknown prototype ID: {:?}", id)
            }
        }
    }
}

impl Error for SpawnObjectError {}

pub fn spawn_store_object_from_prototype(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &ObjectCatalog,
    params: SpawnStoreObjectParams,
) -> Result<Entity, SpawnObjectError> {
    let Some(proto) = catalog.prototypes.get(&params.prototype_id) else {
        return Err(SpawnObjectError::UnknownPrototype(params.prototype_id));
    };
    let placement = match params.placement {
        ObjectPlacement::WallMounted { attachment } => ObjectPlacement::WallMounted {
            attachment: normalize_wall_attachment_for_prototype(proto, attachment),
        },
        floor => floor,
    };

    let image = asset_server.load(&proto.visuals.asset_path);
    let rotation_index = match placement {
        ObjectPlacement::Floor { rotation_index, .. } => rotation_index.unwrap_or(0),
        ObjectPlacement::WallMounted { .. } => 0,
    };
    let is_wall_mounted = matches!(placement, ObjectPlacement::WallMounted { .. });
    let initial_world_pos = match placement {
        ObjectPlacement::Floor { world_pos, .. } => world_pos,
        ObjectPlacement::WallMounted { .. } => Vec2::ZERO,
    };
    let wall_mounted_spec = wall_mounted_spec(proto).cloned();

    let mut entity_commands = commands.spawn((
        (
            Sprite {
                image: image.clone(),
                custom_size: Some(proto.visuals.sprite_size),
                ..default()
            },
            WorldPos(initial_world_pos),
            ProjectedPos::default(),
            FootAnchor(proto.visuals.foot_anchor),
            VisualOffset(Vec2::ZERO),
            if matches!(placement, ObjectPlacement::WallMounted { .. }) {
                SortLayer::WallTopCap
            } else {
                SortLayer::Objects
            },
            SortBias(proto.visuals.sort_bias),
            ObjectPlacementComponent { placement },
        ),
        (
            Interactive,
            Selectable,
            Inspectable,
            Deletable,
            StoreObject,
            InteractionRole::WorldObject,
            ObjectStableId(params.stable_id),
            ObjectPrototypeId(params.prototype_id.clone()),
            Name::new(proto.visuals.asset_id.clone()),
        ),
    ));

    match placement {
        ObjectPlacement::Floor {
            world_pos,
            rotation_index,
        } => {
            entity_commands.insert(FloorPlacement {
                world_pos: WorldPos(world_pos),
                rotation_index,
            });
            entity_commands.insert(FloorFootprint::rectangle(
                proto.placement.footprint_half_extents,
            ));
            entity_commands.insert(Movable);

            if proto.placement.placement_blocker {
                entity_commands.insert(BlocksPlacement);
            }
        }
        ObjectPlacement::WallMounted { attachment } => {
            if let Some(spec) = wall_mounted_spec {
                let wallprint = if let Some(derived) = &params.derived_door {
                    derived.wallprint.clone()
                } else {
                    derive_wallprint(
                        attachment,
                        spec.width,
                        spec.height,
                        wall_occupancy_kind_for_prototype(proto),
                    )
                };
                let bounds = wallprint.rects[0];

                entity_commands.insert(WallMountedPlacement { attachment });
                entity_commands.insert(WallMounted {
                    attachment,
                    width: spec.width,
                    height: spec.height,
                });
                entity_commands.insert(wallprint);
                entity_commands.insert(WallMountedBounds {
                    segment_key: bounds.segment_key,
                    offset_min: bounds.offset_min,
                    offset_max: bounds.offset_max,
                    height_min: bounds.height_min,
                    height_max: bounds.height_max,
                });

                if spec.movable {
                    entity_commands.insert(WallMovable);
                }
            }
        }
    }

    // Map capabilities to components
    for cap in &proto.capabilities {
        match cap {
            ObjectCapabilitySpec::ProductContainer(spec) => {
                entity_commands.insert(ProductContainer {
                    kind: spec.container_kind,
                    capacity_class: spec.capacity_class,
                });
            }
            ObjectCapabilitySpec::CheckoutPoint(spec) => {
                entity_commands.insert(CheckoutPoint {
                    kind: spec.checkout_kind,
                });
            }
            ObjectCapabilitySpec::Decor(spec) => {
                entity_commands.insert(Decor {
                    kind: spec.decor_kind,
                });
            }
            ObjectCapabilitySpec::NpcInteractionPoints(spec) => {
                entity_commands.insert(NpcInteractionPoints {
                    points: spec
                        .points
                        .iter()
                        .map(|p| NpcInteractionPoint {
                            local_pos: p.local_pos,
                            facing: p.facing,
                            kind: p.kind,
                        })
                        .collect(),
                });
            }
            ObjectCapabilitySpec::WallMounted(_) => {}
            ObjectCapabilitySpec::Window(spec) => {
                entity_commands.insert(WallWindow {
                    glass_alpha: spec.glass_alpha,
                });
                entity_commands.insert(Sprite {
                    image: image.clone(),
                    custom_size: Some(proto.visuals.sprite_size),
                    color: Color::srgba(0.75, 0.90, 1.0, spec.glass_alpha),
                    ..default()
                });
            }
            ObjectCapabilitySpec::Doorway(spec) => {
                entity_commands.insert(Doorway { kind: spec.kind });
                if let Some(derived) = &params.derived_door {
                    entity_commands.insert(derived.interior_access_zone.clone());
                }
            }
            ObjectCapabilitySpec::DoorMovable => {
                entity_commands.insert(DoorMovable);
            }
            ObjectCapabilitySpec::WallOpening(spec) => {
                if let ObjectPlacement::WallMounted { attachment } = placement {
                    if let Ok(rect) =
                        crate::store::boundary::opening::derive_opening_rect(
                            attachment.offset_along_segment,
                            attachment.height_on_wall,
                            spec,
                        )
                    {
                        entity_commands.insert(WallOpeningComponent {
                            segment_key: attachment.segment_key,
                            offset_min: rect.offset_min,
                            offset_max: rect.offset_max,
                            height_min: rect.height_min,
                            height_max: rect.height_max,
                            glass_color: spec.glass_color,
                            frame_color: spec.frame_color,
                        });
                    }
                    // Err(UnsupportedShape): catalog validation already rejects Polygon,
                    // so this path is unreachable for valid prototypes. Object spawns
                    // without WallOpeningComponent rather than panicking.
                }
            }
        }
    }

    let entity = entity_commands.id();

    // Setup rotation
    if let Some(mut rotatable) = rotatable_for_prototype(asset_server, proto, image) {
        if rotation_index < rotatable.variants.len() {
            rotatable.current = rotation_index;
            let variant = &rotatable.variants[rotation_index];
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert((
                Sprite {
                    image: variant.sprite.clone(),
                    custom_size: Some(proto.visuals.sprite_size),
                    ..default()
                },
                FootAnchor(variant.foot_anchor),
                VisualOffset(variant.visual_offset),
            ));
            if !is_wall_mounted {
                entity_commands.insert(variant.footprint.clone());
            }
        }
        commands.entity(entity).insert(rotatable);
    }

    Ok(entity)
}

pub fn wall_mounted_spec(proto: &ObjectPrototype) -> Option<&WallMountedSpec> {
    proto.capabilities.iter().find_map(|cap| {
        if let ObjectCapabilitySpec::WallMounted(spec) = cap {
            Some(spec)
        } else {
            None
        }
    })
}

pub fn doorway_spec(proto: &ObjectPrototype) -> Option<&DoorwaySpec> {
    proto.capabilities.iter().find_map(|cap| {
        if let ObjectCapabilitySpec::Doorway(spec) = cap {
            Some(spec)
        } else {
            None
        }
    })
}

pub fn normalize_wall_attachment_for_prototype(
    proto: &ObjectPrototype,
    mut attachment: WallAttachmentPoint,
) -> WallAttachmentPoint {
    if let Some(door_spec) = doorway_spec(proto) {
        attachment.height_on_wall = door_spec.base_height_on_wall.max(0.0);
    }
    attachment
}

pub fn wall_opening_spec(proto: &ObjectPrototype) -> Option<&WallOpeningSpec> {
    proto.capabilities.iter().find_map(|cap| {
        if let ObjectCapabilitySpec::WallOpening(s) = cap {
            Some(s)
        } else {
            None
        }
    })
}

pub fn wall_occupancy_kind_for_prototype(proto: &ObjectPrototype) -> WallOccupancyKind {
    if proto.capabilities.iter().any(|cap| {
        matches!(
            cap,
            ObjectCapabilitySpec::Window(_) | ObjectCapabilitySpec::WallOpening(_)
        )
    }) {
        WallOccupancyKind::Opening
    } else {
        WallOccupancyKind::DecorativeOverlay
    }
}

pub fn spawn_ghost_from_prototype(
    commands: &mut Commands,
    asset_server: &AssetServer,
    proto: &ObjectPrototype,
    world_pos: Vec2,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: asset_server.load(&proto.visuals.asset_path),
                custom_size: Some(proto.visuals.sprite_size),
                color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                ..default()
            },
            WorldPos(world_pos),
            ProjectedPos::default(),
            FootAnchor(proto.visuals.foot_anchor),
            VisualOffset(Vec2::ZERO),
            SortLayer::DragPreview,
            SortBias(proto.visuals.sort_bias),
            Footprint::rectangle(proto.placement.footprint_half_extents),
            BuildGhost,
            GhostOf {
                prototype: proto.id.clone(),
            },
        ))
        .id()
}

pub fn spawn_wall_mounted_preview(
    commands: &mut Commands,
    asset_server: &AssetServer,
    proto: &ObjectPrototype,
    world_pos: Vec2,
    visual_offset: Vec2,
    visible: bool,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: asset_server.load(&proto.visuals.asset_path),
                custom_size: Some(proto.visuals.sprite_size),
                color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                ..default()
            },
            WorldPos(world_pos),
            ProjectedPos::default(),
            FootAnchor(proto.visuals.foot_anchor),
            VisualOffset(visual_offset),
            SortLayer::WallTopCap,
            SortBias(proto.visuals.sort_bias),
            ToolPreview,
            ToolPreviewKind::WallMounted {
                prototype_id: proto.id.clone(),
            },
            PlacementPreview { validation: None },
            NonInteractive,
            WallMountedPreview,
            InteractionRole::ToolPreview,
            RuntimeOwned {
                owner: RuntimeOwner::ToolPreview,
            },
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ))
        .id()
}

fn rotatable_for_prototype(
    asset_server: &AssetServer,
    proto: &ObjectPrototype,
    image: Handle<Image>,
) -> Option<Rotatable> {
    if proto.rotation.kind == RotationKind::None {
        return None;
    }

    let normal = RotationVariant {
        sprite: image.clone(),
        footprint: Footprint::rectangle(proto.placement.footprint_half_extents),
        foot_anchor: proto.visuals.foot_anchor,
        visual_offset: Vec2::ZERO,
    };

    let mut variants = vec![normal];

    if proto.rotation.kind == RotationKind::TwoVariants
        && let Some(rotated_path) = &proto.rotation.rotated_asset_path
    {
        let rotated = RotationVariant {
            sprite: asset_server.load(rotated_path),
            footprint: Footprint::rectangle(Vec2::new(
                proto.placement.footprint_half_extents.y,
                proto.placement.footprint_half_extents.x,
            )),
            foot_anchor: proto.visuals.foot_anchor,
            visual_offset: Vec2::ZERO,
        };
        variants.push(rotated);
    }

    Some(Rotatable {
        current: 0,
        variants,
    })
}

// Startup system to populate catalog
pub fn setup_object_catalog(mut commands: Commands) {
    let mut catalog = ObjectCatalog::default();

    // 1. Shelves
    catalog.prototypes.insert(
        BuildObjectId::new("fixture.shelf.basic"),
        ObjectPrototype {
            id: BuildObjectId::new("fixture.shelf.basic"),
            display: ObjectDisplaySpec {
                display_name: "Basic Shelf".to_string(),
                description: Some("Standard retail shelf for dry goods.".to_string()),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Fixture,
                ribbon_tab: BuildRibbonTab::Fixtures,
                ribbon_group: BuildRibbonGroup::Shelves,
                sort_order: 10,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::Floor,
                footprint_half_extents: Vec2::new(26.0, 18.0),
                placement_blocker: true,
                navigation_blocker: true,
            },
            visuals: VisualSpec {
                asset_path: "chair.png".to_string(), // Reusing asset as placeholder
                asset_id: "shelf_basic".to_string(),
                sprite_size: Vec2::new(96.0, 128.0),
                foot_anchor: Vec2::new(0.0, -48.0),
                sort_bias: -0.2,
            },
            rotation: RotationSpec {
                kind: RotationKind::TwoVariants,
                rotated_asset_path: Some("chair_rotated.png".to_string()),
            },
            capabilities: vec![
                ObjectCapabilitySpec::ProductContainer(ProductContainerSpec {
                    container_kind: ProductContainerKind::Shelf,
                    capacity_class: ContainerCapacityClass::Medium,
                }),
                ObjectCapabilitySpec::NpcInteractionPoints(NpcInteractionPointsSpec {
                    points: vec![NpcInteractionPointSpec {
                        local_pos: Vec2::new(0.0, 32.0),
                        facing: Vec2::new(0.0, -1.0),
                        kind: NpcInteractionKind::BrowseProducts,
                    }],
                }),
            ],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    // 2. Checkout
    catalog.prototypes.insert(
        BuildObjectId::new("service.checkout.basic"),
        ObjectPrototype {
            id: BuildObjectId::new("service.checkout.basic"),
            display: ObjectDisplaySpec {
                display_name: "Basic Checkout".to_string(),
                description: Some("Standard cash register counter.".to_string()),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Service,
                ribbon_tab: BuildRibbonTab::Service,
                ribbon_group: BuildRibbonGroup::Checkout,
                sort_order: 10,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::Floor,
                footprint_half_extents: Vec2::new(54.0, 32.0),
                placement_blocker: true,
                navigation_blocker: true,
            },
            visuals: VisualSpec {
                asset_path: "table.png".to_string(), // Reusing asset
                asset_id: "checkout_basic".to_string(),
                sprite_size: Vec2::new(160.0, 128.0),
                foot_anchor: Vec2::new(0.0, -42.0),
                sort_bias: 0.0,
            },
            rotation: RotationSpec {
                kind: RotationKind::TwoVariants,
                rotated_asset_path: Some("table_rotated.png".to_string()),
            },
            capabilities: vec![
                ObjectCapabilitySpec::CheckoutPoint(CheckoutPointSpec {
                    checkout_kind: CheckoutKind::BasicRegister,
                }),
                ObjectCapabilitySpec::NpcInteractionPoints(NpcInteractionPointsSpec {
                    points: vec![NpcInteractionPointSpec {
                        local_pos: Vec2::new(0.0, 48.0),
                        facing: Vec2::new(0.0, -1.0),
                        kind: NpcInteractionKind::Checkout,
                    }],
                }),
            ],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    // 3. Decor
    catalog.prototypes.insert(
        BuildObjectId::new("decor.plant.tree"),
        ObjectPrototype {
            id: BuildObjectId::new("decor.plant.tree"),
            display: ObjectDisplaySpec {
                display_name: "Ficus Tree".to_string(),
                description: Some("Large indoor plant for decoration.".to_string()),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Decor,
                ribbon_tab: BuildRibbonTab::Decor,
                ribbon_group: BuildRibbonGroup::Decor,
                sort_order: 10,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::Floor,
                footprint_half_extents: Vec2::new(32.0, 28.0),
                placement_blocker: true,
                navigation_blocker: true,
            },
            visuals: VisualSpec {
                asset_path: "tree.png".to_string(),
                asset_id: "tree_ficus".to_string(),
                sprite_size: Vec2::new(144.0, 220.0),
                foot_anchor: Vec2::new(0.0, -86.0),
                sort_bias: 0.2,
            },
            rotation: RotationSpec {
                kind: RotationKind::None,
                rotated_asset_path: None,
            },
            capabilities: vec![ObjectCapabilitySpec::Decor(DecorSpec {
                decor_kind: DecorKind::Plant,
            })],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    // 4. Wall decor and visual-only window placeholders for Stage 5B.
    catalog.prototypes.insert(
        BuildObjectId::new("wall.decor.placeholder"),
        ObjectPrototype {
            id: BuildObjectId::new("wall.decor.placeholder"),
            display: ObjectDisplaySpec {
                display_name: "Wall Decor".to_string(),
                description: Some("Internal wall-mounted preview test object.".to_string()),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Decor,
                ribbon_tab: BuildRibbonTab::Walls,
                ribbon_group: BuildRibbonGroup::Walls,
                sort_order: 999,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::WallMounted,
                footprint_half_extents: Vec2::new(20.0, 14.0),
                placement_blocker: false,
                navigation_blocker: false,
            },
            visuals: VisualSpec {
                asset_path: "tree.png".to_string(),
                asset_id: "wall_decor_placeholder".to_string(),
                sprite_size: Vec2::new(64.0, 64.0),
                foot_anchor: Vec2::new(0.0, -24.0),
                sort_bias: 0.0,
            },
            rotation: RotationSpec {
                kind: RotationKind::None,
                rotated_asset_path: None,
            },
            capabilities: vec![
                ObjectCapabilitySpec::Decor(DecorSpec {
                    decor_kind: DecorKind::Misc,
                }),
                ObjectCapabilitySpec::WallMounted(WallMountedSpec {
                    width: 64.0,
                    height: 64.0,
                    allowed_sides: vec![
                        crate::store::StoreBoundarySide::Top,
                        crate::store::StoreBoundarySide::Right,
                    ],
                    default_height_on_wall: 48.0,
                    movable: true,
                }),
            ],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    catalog.prototypes.insert(
        BuildObjectId::new("wall.window.basic_visual"),
        ObjectPrototype {
            id: BuildObjectId::new("wall.window.basic_visual"),
            display: ObjectDisplaySpec {
                display_name: "Basic Window".to_string(),
                description: Some(
                    "Visual-only wall window. No cutout, collision, or navigation semantics."
                        .to_string(),
                ),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Decor,
                ribbon_tab: BuildRibbonTab::Walls,
                ribbon_group: BuildRibbonGroup::Walls,
                sort_order: 1000,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::WallMounted,
                footprint_half_extents: Vec2::new(28.0, 18.0),
                placement_blocker: false,
                navigation_blocker: false,
            },
            visuals: VisualSpec {
                asset_path: "tree.png".to_string(),
                asset_id: "wall_window_basic_visual".to_string(),
                sprite_size: Vec2::new(72.0, 72.0),
                foot_anchor: Vec2::new(0.0, -36.0),
                sort_bias: 0.1,
            },
            rotation: RotationSpec {
                kind: RotationKind::None,
                rotated_asset_path: None,
            },
            capabilities: vec![
                ObjectCapabilitySpec::Decor(DecorSpec {
                    decor_kind: DecorKind::Misc,
                }),
                ObjectCapabilitySpec::WallMounted(WallMountedSpec {
                    width: 72.0,
                    height: 72.0,
                    allowed_sides: vec![
                        crate::store::StoreBoundarySide::Top,
                        crate::store::StoreBoundarySide::Right,
                    ],
                    default_height_on_wall: 56.0,
                    movable: true,
                }),
                ObjectCapabilitySpec::Window(WindowSpec {
                    width: 72.0,
                    height: 72.0,
                    glass_alpha: 0.45,
                }),
                ObjectCapabilitySpec::WallOpening(WallOpeningSpec {
                    shape: WallOpeningShapeSpec::Rect {
                        width: 72.0,
                        height: 72.0,
                        anchor: WallOpeningAnchor::BottomCenter,
                    },
                    glass_color: Some(Color::srgba(0.75, 0.90, 1.0, 0.45)),
                    frame_color: None,
                }),
            ],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    catalog.prototypes.insert(
        BuildObjectId::new("wall.door.basic_customer"),
        ObjectPrototype {
            id: BuildObjectId::new("wall.door.basic_customer"),
            display: ObjectDisplaySpec {
                display_name: "Basic Door".to_string(),
                description: Some("Customer entrance door.".to_string()),
                icon: None,
            },
            catalog: ObjectCatalogSpec {
                category: ObjectCategory::Decor,
                ribbon_tab: BuildRibbonTab::Walls,
                ribbon_group: BuildRibbonGroup::Walls,
                sort_order: 2000,
                availability: CatalogAvailability::Available,
            },
            placement: PlacementSpec {
                kind: PlacementKind::WallMounted,
                footprint_half_extents: Vec2::new(32.0, 10.0),
                placement_blocker: false,
                navigation_blocker: false,
            },
            visuals: VisualSpec {
                asset_path: "tree.png".to_string(),
                asset_id: "wall_door_basic_customer".to_string(),
                sprite_size: Vec2::new(64.0, 96.0),
                foot_anchor: Vec2::new(0.0, -32.0),
                sort_bias: 0.1,
            },
            rotation: RotationSpec {
                kind: RotationKind::None,
                rotated_asset_path: None,
            },
            capabilities: vec![
                ObjectCapabilitySpec::WallMounted(WallMountedSpec {
                    width: 64.0,
                    height: 96.0,
                    allowed_sides: vec![
                        crate::store::StoreBoundarySide::Top,
                        crate::store::StoreBoundarySide::Right,
                    ],
                    default_height_on_wall: 0.0,
                    movable: false, // Use DoorMovable instead
                }),
                ObjectCapabilitySpec::Doorway(DoorwaySpec {
                    access_width: 64.0,
                    access_depth: 64.0,
                    base_height_on_wall: 0.0,
                    kind: crate::objects::components::DoorwayKind::CustomerEntrance,
                }),
                ObjectCapabilitySpec::DoorMovable,
                ObjectCapabilitySpec::WallOpening(WallOpeningSpec {
                    shape: WallOpeningShapeSpec::Rect {
                        width: 64.0,
                        height: 96.0,
                        anchor: WallOpeningAnchor::BottomCenter,
                    },
                    glass_color: None,
                    frame_color: None,
                }),
            ],
            initial_state: ObjectInitialStateSpec::None,
        },
    );

    // 4. Legacy Aliases for Save/Load compat
    let shelf = catalog
        .prototypes
        .get(&BuildObjectId::new("fixture.shelf.basic"))
        .unwrap()
        .clone();
    catalog
        .prototypes
        .insert(BuildObjectId::new("chair"), shelf);

    let checkout = catalog
        .prototypes
        .get(&BuildObjectId::new("service.checkout.basic"))
        .unwrap()
        .clone();
    catalog
        .prototypes
        .insert(BuildObjectId::new("table"), checkout);

    let tree = catalog
        .prototypes
        .get(&BuildObjectId::new("decor.plant.tree"))
        .unwrap()
        .clone();
    catalog.prototypes.insert(BuildObjectId::new("tree"), tree);

    let report = validate_object_catalog(&catalog);
    if !report.is_empty() {
        error!("Catalog validation failed:\n{}", report.join("\n"));
    }

    commands.insert_resource(catalog);
}

pub fn validate_object_catalog(catalog: &ObjectCatalog) -> Vec<String> {
    let mut errors = Vec::new();
    for proto in catalog.prototypes.values() {
        if proto.display.display_name.is_empty() {
            errors.push(format!("Prototype {:?} has empty display name", proto.id));
        }

        // Capability invariants
        for cap in &proto.capabilities {
            match cap {
                ObjectCapabilitySpec::ProductContainer(_) => {
                    let has_browse = proto.capabilities.iter().any(|c| {
                        if let ObjectCapabilitySpec::NpcInteractionPoints(p) = c {
                            p.points
                                .iter()
                                .any(|pt| pt.kind == NpcInteractionKind::BrowseProducts)
                        } else {
                            false
                        }
                    });
                    if !has_browse {
                        errors.push(format!("Prototype {:?} has ProductContainer but no BrowseProducts interaction point", proto.id));
                    }
                }
                ObjectCapabilitySpec::CheckoutPoint(_) => {
                    let has_checkout = proto.capabilities.iter().any(|c| {
                        if let ObjectCapabilitySpec::NpcInteractionPoints(p) = c {
                            p.points
                                .iter()
                                .any(|pt| pt.kind == NpcInteractionKind::Checkout)
                        } else {
                            false
                        }
                    });
                    if !has_checkout {
                        errors.push(format!(
                            "Prototype {:?} has CheckoutPoint but no Checkout interaction point",
                            proto.id
                        ));
                    }
                }
                ObjectCapabilitySpec::WallMounted(spec) => {
                    if proto.placement.kind != PlacementKind::WallMounted {
                        errors.push(format!(
                            "Prototype {:?} has WallMounted capability but non-wall placement",
                            proto.id
                        ));
                    }
                    if spec.width <= 0.0 || spec.height <= 0.0 {
                        errors.push(format!(
                            "Prototype {:?} has invalid WallMounted size",
                            proto.id
                        ));
                    }
                    if spec.default_height_on_wall < 0.0 {
                        errors.push(format!(
                            "Prototype {:?} has invalid default wall mount height",
                            proto.id
                        ));
                    }
                    if spec.allowed_sides.is_empty() {
                        errors.push(format!(
                            "Prototype {:?} has no allowed wall sides",
                            proto.id
                        ));
                    }
                }
                ObjectCapabilitySpec::Window(spec) => {
                    if proto.placement.kind != PlacementKind::WallMounted {
                        errors.push(format!(
                            "Prototype {:?} has Window capability but is not wall-mounted",
                            proto.id
                        ));
                    }
                    if spec.width <= 0.0 || spec.height <= 0.0 {
                        errors.push(format!("Prototype {:?} has invalid Window size", proto.id));
                    }
                    if !(0.0..=1.0).contains(&spec.glass_alpha) {
                        errors.push(format!(
                            "Prototype {:?} has invalid Window glass alpha",
                            proto.id
                        ));
                    }
                    if wall_mounted_spec(proto).is_none() {
                        errors.push(format!(
                            "Prototype {:?} has Window capability without WallMounted spec",
                            proto.id
                        ));
                    }
                }
                ObjectCapabilitySpec::WallOpening(spec) => {
                    if proto.placement.kind != PlacementKind::WallMounted {
                        errors.push(format!(
                            "Prototype {:?} has WallOpening capability but is not wall-mounted",
                            proto.id
                        ));
                    }
                    if wall_mounted_spec(proto).is_none() {
                        errors.push(format!(
                            "Prototype {:?} has WallOpening capability without WallMounted spec",
                            proto.id
                        ));
                    }
                    match &spec.shape {
                        WallOpeningShapeSpec::Rect { width, height, .. } => {
                            if *width <= 0.0 || *height <= 0.0 {
                                errors.push(format!(
                                    "Prototype {:?} has WallOpening with zero or negative Rect dimensions",
                                    proto.id
                                ));
                            }
                        }
                        WallOpeningShapeSpec::Polygon { .. } => {
                            errors.push(format!(
                                "Prototype {:?} has WallOpening with Polygon shape, which is not supported in Stage 5B.6",
                                proto.id
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests;
