use bevy::prelude::*;
use std::collections::HashSet;

use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{
    StoreArea, StoreChunkCoord, StoreChunkKind, WorldBounds, side_neighbors,
    validate_chunk_purchase,
};
use crate::tools::{ExpansionToolState, ToolMode};

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct StoreChunkOverlay {
    pub coord: StoreChunkCoord,
    pub kind: StoreChunkOverlayKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreChunkOverlayKind {
    Owned,
    Available,
    HoveredAvailable,
}

#[derive(Component, Debug, Clone, Copy)]
struct StoreChunkOverlaySegment;

pub struct StoreOverlayPlugin;

impl Plugin for StoreOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_store_chunk_overlays.before(TransformSystems::Propagate),
        );
    }
}

fn update_store_chunk_overlays(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    store: Option<Res<StoreArea>>,
    world: Option<Res<WorldBounds>>,
    expansion: Res<ExpansionToolState>,
    projection: Res<IsoProjection>,
    overlays: Query<Entity, With<StoreChunkOverlaySegment>>,
) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }

    let (Some(store), Some(world)) = (store, world) else {
        return;
    };

    for coord in store.owned_chunks.keys().copied() {
        spawn_chunk_outline(
            &mut commands,
            &store,
            coord,
            StoreChunkOverlayKind::Owned,
            Color::srgba(0.20, 0.62, 0.55, 0.45),
            3.0,
            920.0,
            *projection,
        );
    }

    if *mode.get() != ToolMode::Expansion {
        return;
    }

    for coord in available_expansion_chunks(&world, &store) {
        let (kind, color, thickness, z) =
            if expansion.hovered_chunk == Some(coord) && expansion.hovered_valid {
                (
                    StoreChunkOverlayKind::HoveredAvailable,
                    Color::srgba(1.0, 0.86, 0.20, 0.88),
                    7.0,
                    945.0,
                )
            } else {
                (
                    StoreChunkOverlayKind::Available,
                    Color::srgba(0.38, 0.78, 0.42, 0.58),
                    5.0,
                    935.0,
                )
            };
        spawn_chunk_outline(
            &mut commands,
            &store,
            coord,
            kind,
            color,
            thickness,
            z,
            *projection,
        );
    }
}

fn available_expansion_chunks(world: &WorldBounds, store: &StoreArea) -> Vec<StoreChunkCoord> {
    let mut candidates = HashSet::new();
    for coord in store.owned_chunks.keys().copied() {
        for neighbor in side_neighbors(coord) {
            candidates.insert(neighbor);
        }
    }

    let mut valid: Vec<_> = candidates
        .into_iter()
        .filter(|coord| {
            validate_chunk_purchase(world, store, *coord, StoreChunkKind::Default).is_ok()
        })
        .collect();
    valid.sort_by_key(|coord| (coord.y, coord.x));
    valid
}

#[allow(clippy::too_many_arguments)]
fn spawn_chunk_outline(
    commands: &mut Commands,
    store: &StoreArea,
    coord: StoreChunkCoord,
    kind: StoreChunkOverlayKind,
    color: Color,
    thickness: f32,
    z: f32,
    projection: IsoProjection,
) {
    let rect = store.chunk_rect(coord);
    let points = [
        rect.min,
        Vec2::new(rect.max.x, rect.min.y),
        rect.max,
        Vec2::new(rect.min.x, rect.max.y),
    ];

    for (a, b) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let pa = world_to_iso(a, projection);
        let pb = world_to_iso(b, projection);
        let delta = pb - pa;
        let length = delta.length();
        if length <= 0.1 {
            continue;
        }
        let mid = (pa + pb) * 0.5;
        commands.spawn((
            Sprite::from_color(color, Vec2::new(length, thickness)),
            Transform {
                translation: Vec3::new(mid.x, mid.y, z),
                rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                ..default()
            },
            Visibility::Visible,
            StoreChunkOverlay { coord, kind },
            StoreChunkOverlaySegment,
            Name::new(format!("StoreChunkOverlay {:?} {:?}", kind, coord)),
        ));
    }
}
