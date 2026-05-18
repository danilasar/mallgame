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
}

#[derive(Debug, Clone)]
pub struct WallMountedSpec {
    pub width: f32,
    pub height: f32,
    pub allowed_sides: Vec<crate::store::StoreBoundarySide>,
    pub default_height_on_wall: f32,
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

    let image = asset_server.load(&proto.visuals.asset_path);
    let rotation_index = match params.placement {
        ObjectPlacement::Floor { rotation_index, .. } => rotation_index.unwrap_or(0),
        ObjectPlacement::WallMounted { .. } => 0,
    };
    let is_wall_mounted = matches!(params.placement, ObjectPlacement::WallMounted { .. });
    let initial_world_pos = match params.placement {
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
            if matches!(params.placement, ObjectPlacement::WallMounted { .. }) {
                SortLayer::WallTopCap
            } else {
                SortLayer::Objects
            },
            SortBias(proto.visuals.sort_bias),
            ObjectPlacementComponent {
                placement: params.placement,
            },
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

    if !is_wall_mounted {
        entity_commands.insert(Footprint::rectangle(proto.placement.footprint_half_extents));
    }

    if matches!(params.placement, ObjectPlacement::Floor { .. }) {
        entity_commands.insert(Movable);
    }

    if matches!(params.placement, ObjectPlacement::Floor { .. })
        && proto.placement.placement_blocker
    {
        entity_commands.insert(BlocksPlacement);
    }

    if let (ObjectPlacement::WallMounted { attachment }, Some(spec)) =
        (params.placement, wall_mounted_spec)
    {
        entity_commands.insert(WallMounted {
            attachment,
            width: spec.width,
            height: spec.height,
        });
        entity_commands.insert(WallMountedBounds {
            segment_key: attachment.segment_key,
            offset_min: attachment.offset_along_segment - spec.width * 0.5,
            offset_max: attachment.offset_along_segment + spec.width * 0.5,
            height_min: attachment.height_on_wall,
            height_max: attachment.height_on_wall + spec.height,
        });
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
        }
    }

    let entity = entity_commands.id();

    // Setup rotation
    if let Some(mut rotatable) = rotatable_for_prototype(asset_server, proto, image) {
        if rotation_index < rotatable.variants.len() {
            rotatable.current = rotation_index;
            let variant = &rotatable.variants[rotation_index];
            commands.entity(entity).insert((
                Sprite {
                    image: variant.sprite.clone(),
                    custom_size: Some(proto.visuals.sprite_size),
                    ..default()
                },
                variant.footprint.clone(),
                FootAnchor(variant.foot_anchor),
                VisualOffset(variant.visual_offset),
            ));
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
                foot_anchor: Vec2::new(0.0, -24.0),
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
                }),
                ObjectCapabilitySpec::Window(WindowSpec {
                    width: 72.0,
                    height: 72.0,
                    glass_alpha: 0.45,
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
                _ => {}
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_validation_missing_browse_point() {
        let mut catalog = ObjectCatalog::default();
        catalog.prototypes.insert(
            BuildObjectId::new("fail"),
            ObjectPrototype {
                id: BuildObjectId::new("fail"),
                display: ObjectDisplaySpec {
                    display_name: "Fail".to_string(),
                    description: None,
                    icon: None,
                },
                catalog: ObjectCatalogSpec {
                    category: ObjectCategory::Fixture,
                    ribbon_tab: BuildRibbonTab::Fixtures,
                    ribbon_group: BuildRibbonGroup::Shelves,
                    sort_order: 0,
                    availability: CatalogAvailability::Available,
                },
                placement: PlacementSpec {
                    kind: PlacementKind::Floor,
                    footprint_half_extents: Vec2::ZERO,
                    placement_blocker: false,
                    navigation_blocker: false,
                },
                visuals: VisualSpec {
                    asset_path: "".into(),
                    asset_id: "".into(),
                    sprite_size: Vec2::ZERO,
                    foot_anchor: Vec2::ZERO,
                    sort_bias: 0.0,
                },
                rotation: RotationSpec {
                    kind: RotationKind::None,
                    rotated_asset_path: None,
                },
                capabilities: vec![ObjectCapabilitySpec::ProductContainer(
                    ProductContainerSpec {
                        container_kind: ProductContainerKind::Shelf,
                        capacity_class: ContainerCapacityClass::Small,
                    },
                )],
                initial_state: ObjectInitialStateSpec::None,
            },
        );

        let errors = validate_object_catalog(&catalog);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("no BrowseProducts interaction point"))
        );
    }

    #[test]
    fn test_factory_mapping_capabilities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        crate::store::commands::register_test_messages(&mut app);

        // Re-setup standard catalog for testing
        let commands = app.world_mut().commands();
        setup_object_catalog(commands);
        app.update();

        let catalog = app.world().resource::<ObjectCatalog>().clone();
        let asset_server = app.world().resource::<AssetServer>().clone();

        let mut commands = app.world_mut().commands();
        let proto_id = BuildObjectId::new("fixture.shelf.basic");
        let entity = spawn_store_object_from_prototype(
            &mut commands,
            &asset_server,
            &catalog,
            SpawnStoreObjectParams {
                stable_id: StableObjectId(1),
                prototype_id: proto_id.clone(),
                placement: ObjectPlacement::Floor {
                    world_pos: Vec2::ZERO,
                    rotation_index: None,
                },
            },
        )
        .expect("Spawn failed");

        app.update();

        let world = app.world();
        assert!(world.entity(entity).contains::<ProductContainer>());
        assert!(world.entity(entity).contains::<NpcInteractionPoints>());
        assert!(!world.entity(entity).contains::<CheckoutPoint>());
        assert!(world.entity(entity).contains::<StoreObject>());
        assert!(world.entity(entity).contains::<Footprint>());
        assert!(world.entity(entity).contains::<Movable>());
    }

    #[test]
    fn test_wall_mounted_factory_does_not_add_floor_geometry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();

        let commands = app.world_mut().commands();
        setup_object_catalog(commands);
        app.update();

        let catalog = app.world().resource::<ObjectCatalog>().clone();
        let asset_server = app.world().resource::<AssetServer>().clone();
        let segment_key = crate::store::WallSegmentKey {
            chunk: crate::store::StoreChunkCoord { x: -1, y: -1 },
            side: crate::store::StoreBoundarySide::Top,
        };

        let mut commands = app.world_mut().commands();
        let entity = spawn_store_object_from_prototype(
            &mut commands,
            &asset_server,
            &catalog,
            SpawnStoreObjectParams {
                stable_id: StableObjectId(2),
                prototype_id: BuildObjectId::new("wall.decor.placeholder"),
                placement: ObjectPlacement::WallMounted {
                    attachment: WallAttachmentPoint {
                        segment_key,
                        offset_along_segment: 64.0,
                        height_on_wall: 48.0,
                    },
                },
            },
        )
        .expect("Spawn failed");

        app.update();

        let world = app.world();
        assert!(world.entity(entity).contains::<StoreObject>());
        assert!(world.entity(entity).contains::<WallMounted>());
        assert!(world.entity(entity).contains::<WallMountedBounds>());
        assert!(!world.entity(entity).contains::<Footprint>());
        assert!(!world.entity(entity).contains::<BlocksPlacement>());
        assert!(!world.entity(entity).contains::<Movable>());
    }

    #[test]
    fn test_wall_mounted_prototype_is_visible_in_walls_tab() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        let commands = app.world_mut().commands();
        setup_object_catalog(commands);
        app.update();

        let catalog = app.world().resource::<ObjectCatalog>();
        let proto = catalog
            .prototypes
            .get(&BuildObjectId::new("wall.decor.placeholder"))
            .expect("wall decor prototype should exist");

        assert_eq!(proto.placement.kind, PlacementKind::WallMounted);
        assert_eq!(proto.catalog.ribbon_tab, BuildRibbonTab::Walls);
        assert_eq!(proto.catalog.availability, CatalogAvailability::Available);

        let window = catalog
            .prototypes
            .get(&BuildObjectId::new("wall.window.basic_visual"))
            .expect("visual window prototype should exist");

        assert_eq!(window.placement.kind, PlacementKind::WallMounted);
        assert_eq!(window.catalog.ribbon_tab, BuildRibbonTab::Walls);
        assert!(
            window
                .capabilities
                .iter()
                .any(|cap| matches!(cap, ObjectCapabilitySpec::Window(WindowSpec { .. })))
        );
    }
}
