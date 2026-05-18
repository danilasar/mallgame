use bevy::prelude::*;
use std::collections::HashSet;

use crate::objects::components::{InteractionRole, RuntimeOwned, RuntimeOwner, SortLayer};
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{
    StoreArea, StoreChunkCoord, StoreChunkKind, WorldBounds, side_neighbors,
    validate_chunk_purchase,
};
use crate::tools::{ActiveToolSession, NonInteractive, ToolMode, ToolSessionState};
use bevy::ecs::system::SystemParam;

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

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
struct StoreOverlayParams<'w, 's> {
    commands: Commands<'w, 's>,
    mode: Res<'w, State<ToolMode>>,
    store: Option<Res<'w, StoreArea>>,
    world: Option<Res<'w, WorldBounds>>,
    session: Res<'w, ToolSessionState>,
    projection: Res<'w, IsoProjection>,
    overlays: Query<'w, 's, Entity, With<StoreChunkOverlaySegment>>,
}

fn update_store_chunk_overlays(mut params: StoreOverlayParams) {
    for entity in &params.overlays {
        params.commands.entity(entity).despawn();
    }

    let (Some(store), Some(world)) = (params.store.as_deref(), params.world.as_deref()) else {
        return;
    };

    // Отрисовка купленной территории как "Белой сетки"
    for coord in store.owned_chunks.keys().copied() {
        spawn_chunk_outline(
            &mut params.commands,
            store,
            coord,
            StoreChunkOverlayKind::Owned,
            Color::srgba(1.0, 1.0, 1.0, 0.25),
            2.0,
            SortLayer::StoreOverlay.base_z(),
            *params.projection,
        );
    }

    if *params.mode.get() != ToolMode::Expansion {
        return;
    }

    let (hovered_chunk, hovered_valid) =
        if let Some(ActiveToolSession::Expansion(exp)) = &params.session.active {
            (
                exp.hovered_coord,
                exp.validation.as_ref().is_some_and(|v| v.valid),
            )
        } else {
            (None, false)
        };

    for coord in available_expansion_chunks(world, store) {
        let (kind, color, thickness, z) = if hovered_chunk == Some(coord) && hovered_valid {
            (
                StoreChunkOverlayKind::HoveredAvailable,
                Color::srgba(1.0, 0.86, 0.20, 0.88),
                6.0,
                SortLayer::StoreOverlay.base_z() + 20.0,
            )
        } else {
            (
                StoreChunkOverlayKind::Available,
                Color::srgba(1.0, 1.0, 1.0, 0.15),
                4.0,
                SortLayer::StoreOverlay.base_z() + 10.0,
            )
        };
        spawn_chunk_outline(
            &mut params.commands,
            store,
            coord,
            kind,
            color,
            thickness,
            z,
            *params.projection,
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
            validate_chunk_purchase(world, store, *coord, StoreChunkKind::Default).valid
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
        let owner = if kind == StoreChunkOverlayKind::Owned {
            RuntimeOwner::StoreOverlay
        } else {
            RuntimeOwner::ExpansionOverlay
        };

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
            InteractionRole::Overlay,
            RuntimeOwned { owner },
            NonInteractive,
            Name::new(format!("StoreChunkOverlay {:?} {:?}", kind, coord)),
        ));
    }
}
