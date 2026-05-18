use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::components::*;
use super::rotation::{Rotatable, RotationVariant};

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
    Store,
}

impl BuildRibbonTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fixtures => "Fixtures",
            Self::Service => "Service",
            Self::Decor => "Decor",
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
    pub world_pos: Vec2,
    pub rotation_index: Option<usize>,
}

pub fn spawn_store_object_from_prototype(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &ObjectCatalog,
    params: SpawnStoreObjectParams,
) -> Result<Entity, String> {
    let Some(proto) = catalog.prototypes.get(&params.prototype_id) else {
        return Err(format!("Unknown prototype ID: {:?}", params.prototype_id));
    };

    let image = asset_server.load(&proto.visuals.asset_path);
    let rotation_index = params.rotation_index.unwrap_or(0);

    let mut entity_commands = commands.spawn((
        (
            Sprite {
                image: image.clone(),
                custom_size: Some(proto.visuals.sprite_size),
                ..default()
            },
            WorldPos(params.world_pos),
            ProjectedPos::default(),
            FootAnchor(proto.visuals.foot_anchor),
            VisualOffset(Vec2::ZERO),
            SortLayer::Objects,
            SortBias(proto.visuals.sort_bias),
            Footprint::rectangle(proto.placement.footprint_half_extents),
        ),
        (
            Interactive,
            Selectable,
            Inspectable,
            Movable,
            Deletable,
            StoreObject,
            InteractionRole::WorldObject,
            ObjectStableId(params.stable_id),
            ObjectPrototypeId(params.prototype_id.clone()),
            PlaceableAssetId(Box::leak(proto.visuals.asset_id.clone().into_boxed_str())), // Legacy compat
            Name::new(proto.visuals.asset_id.clone()),
        ),
    ));

    if proto.placement.placement_blocker {
        entity_commands.insert(BlocksPlacement);
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

    if proto.rotation.kind == RotationKind::TwoVariants {
        if let Some(rotated_path) = &proto.rotation.rotated_asset_path {
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
                _ => {}
            }
        }
    }
    errors
}

// Legacy compat markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum BuildPrototypeId {
    Chair,
    Table,
    Tree,
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
                world_pos: Vec2::ZERO,
                rotation_index: None,
            },
        )
        .expect("Spawn failed");

        app.update();

        let world = app.world();
        assert!(world.entity(entity).contains::<ProductContainer>());
        assert!(world.entity(entity).contains::<NpcInteractionPoints>());
        assert!(!world.entity(entity).contains::<CheckoutPoint>());
        assert!(world.entity(entity).contains::<StoreObject>());
    }
}
