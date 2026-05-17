use bevy::prelude::*;

use super::components::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildPrototypeId {
    Chair,
    Table,
    Tree,
}

#[derive(Resource, Debug)]
pub struct BuildPrototypes {
    pub active: BuildPrototypeId,
}

impl Default for BuildPrototypes {
    fn default() -> Self {
        Self {
            active: BuildPrototypeId::Chair,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrototypeSpec {
    pub asset_path: &'static str,
    pub asset_id: &'static str,
    pub sprite_size: Vec2,
    pub foot_anchor: Vec2,
    pub footprint_half_extents: Vec2,
    pub sort_bias: f32,
}

pub fn prototype_spec(prototype: BuildPrototypeId) -> PrototypeSpec {
    match prototype {
        BuildPrototypeId::Chair => PrototypeSpec {
            asset_path: "chair.png",
            asset_id: "chair",
            sprite_size: Vec2::new(96.0, 128.0),
            foot_anchor: Vec2::new(0.0, -48.0),
            footprint_half_extents: Vec2::new(26.0, 18.0),
            sort_bias: -0.2,
        },
        BuildPrototypeId::Table => PrototypeSpec {
            asset_path: "table.png",
            asset_id: "table",
            sprite_size: Vec2::new(160.0, 128.0),
            foot_anchor: Vec2::new(0.0, -42.0),
            footprint_half_extents: Vec2::new(54.0, 32.0),
            sort_bias: 0.0,
        },
        BuildPrototypeId::Tree => PrototypeSpec {
            asset_path: "tree.png",
            asset_id: "tree",
            sprite_size: Vec2::new(144.0, 220.0),
            foot_anchor: Vec2::new(0.0, -86.0),
            footprint_half_extents: Vec2::new(32.0, 28.0),
            sort_bias: 0.2,
        },
    }
}

pub fn spawn_object_from_prototype(
    commands: &mut Commands,
    asset_server: &AssetServer,
    prototype: BuildPrototypeId,
    world_pos: Vec2,
) -> Entity {
    let spec = prototype_spec(prototype);

    commands
        .spawn((
            Sprite {
                image: asset_server.load(spec.asset_path),
                custom_size: Some(spec.sprite_size),
                ..default()
            },
            WorldPos(world_pos),
            Velocity::default(),
            ProjectedPos::default(),
            FootAnchor(spec.foot_anchor),
            VisualOffset(Vec2::ZERO),
            SortLayer::Objects,
            SortBias(spec.sort_bias),
            Footprint::rectangle(spec.footprint_half_extents),
            BlocksPlacement,
            Interactive,
            Movable,
            Deletable,
            PlaceableAssetId(spec.asset_id),
        ))
        .id()
}

pub fn spawn_ghost_from_prototype(
    commands: &mut Commands,
    asset_server: &AssetServer,
    prototype: BuildPrototypeId,
    world_pos: Vec2,
) -> Entity {
    let spec = prototype_spec(prototype);

    commands
        .spawn((
            Sprite {
                image: asset_server.load(spec.asset_path),
                custom_size: Some(spec.sprite_size),
                color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                ..default()
            },
            WorldPos(world_pos),
            ProjectedPos::default(),
            FootAnchor(spec.foot_anchor),
            VisualOffset(Vec2::ZERO),
            SortLayer::DragPreview,
            SortBias(spec.sort_bias),
            Footprint::rectangle(spec.footprint_half_extents),
            BuildGhost,
            GhostOf { prototype },
            PlaceableAssetId(spec.asset_id),
        ))
        .id()
}
