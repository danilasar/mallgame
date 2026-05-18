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
use std::collections::HashMap;

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

#[derive(Resource, Default)]
struct StoreOverlayCache {
    owned: HashMap<StoreChunkCoord, Vec<Entity>>,
    available: HashMap<StoreChunkCoord, Vec<Entity>>,
    expansion_visible: bool,
    last_hovered_available: Option<StoreChunkCoord>,
    initialized: bool,
}

pub struct StoreOverlayPlugin;

impl Plugin for StoreOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StoreOverlayCache>().add_systems(
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
    cache: ResMut<'w, StoreOverlayCache>,
}

fn update_store_chunk_overlays(mut params: StoreOverlayParams) {
    let (Some(store), Some(world)) = (params.store.as_deref(), params.world.as_deref()) else {
        clear_overlay_cache(&mut params.commands, &mut params.cache);
        return;
    };

    let expansion_visible = *params.mode.get() == ToolMode::Expansion;
    let current_hovered = if expansion_visible {
        if let Some(ActiveToolSession::Expansion(exp)) = &params.session.active {
            exp.hovered_coord
        } else {
            None
        }
    } else {
        None
    };

    let store_changed = params
        .store
        .as_ref()
        .is_some_and(|store| store.is_changed());
    let world_changed = params
        .world
        .as_ref()
        .is_some_and(|world| world.is_changed());

    if !params.cache.initialized || store_changed || world_changed {
        rebuild_overlay_cache(
            &mut params.commands,
            &mut params.cache,
            store,
            world,
            *params.projection,
            expansion_visible,
            current_hovered,
        );
        return;
    }

    if params.cache.expansion_visible != expansion_visible {
        set_map_visibility(
            &mut params.commands,
            &params.cache.available,
            expansion_visible,
        );
        params.cache.expansion_visible = expansion_visible;
        params.cache.last_hovered_available = None;
    }

    if !expansion_visible {
        return;
    }

    if params.cache.last_hovered_available != current_hovered {
        if let Some(prev) = params.cache.last_hovered_available {
            refresh_available_coord(
                &mut params.commands,
                RefreshAvailableCoordSpec {
                    cache: &mut params.cache,
                    store,
                    world,
                    projection: *params.projection,
                    coord: prev,
                    hovered: false,
                    visible: true,
                },
            );
        }
        if let Some(current) = current_hovered {
            refresh_available_coord(
                &mut params.commands,
                RefreshAvailableCoordSpec {
                    cache: &mut params.cache,
                    store,
                    world,
                    projection: *params.projection,
                    coord: current,
                    hovered: true,
                    visible: true,
                },
            );
        }
        params.cache.last_hovered_available = current_hovered;
    }
}

fn clear_overlay_cache(commands: &mut Commands, cache: &mut StoreOverlayCache) {
    for entities in cache.owned.values() {
        for entity in entities {
            commands.entity(*entity).despawn();
        }
    }
    for entities in cache.available.values() {
        for entity in entities {
            commands.entity(*entity).despawn();
        }
    }
    cache.owned.clear();
    cache.available.clear();
    cache.last_hovered_available = None;
    cache.expansion_visible = false;
    cache.initialized = false;
}

fn rebuild_overlay_cache(
    commands: &mut Commands,
    cache: &mut StoreOverlayCache,
    store: &StoreArea,
    world: &WorldBounds,
    projection: IsoProjection,
    expansion_visible: bool,
    hovered: Option<StoreChunkCoord>,
) {
    clear_overlay_cache(commands, cache);

    let mut owned_coords: Vec<_> = store.owned_chunks.keys().copied().collect();
    owned_coords.sort_by_key(|coord| (coord.y, coord.x));
    for coord in owned_coords {
        let entities = spawn_chunk_outline(
            commands,
            ChunkOutlineSpec {
                store,
                coord,
                kind: StoreChunkOverlayKind::Owned,
                color: Color::srgba(1.0, 1.0, 1.0, 0.25),
                thickness: 2.0,
                z: SortLayer::StoreOverlay.base_z(),
                projection,
                visible: true,
            },
        );
        cache.owned.insert(coord, entities);
    }

    let mut available_coords = available_expansion_chunks(world, store);
    available_coords.sort_by_key(|coord| (coord.y, coord.x));
    for coord in available_coords {
        let hovered_now = expansion_visible && hovered == Some(coord);
        let entities = spawn_chunk_outline(
            commands,
            ChunkOutlineSpec {
                store,
                coord,
                kind: if hovered_now {
                    StoreChunkOverlayKind::HoveredAvailable
                } else {
                    StoreChunkOverlayKind::Available
                },
                color: if hovered_now {
                    Color::srgba(1.0, 0.86, 0.20, 0.88)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.15)
                },
                thickness: if hovered_now { 6.0 } else { 4.0 },
                z: if hovered_now {
                    SortLayer::StoreOverlay.base_z() + 20.0
                } else {
                    SortLayer::StoreOverlay.base_z() + 10.0
                },
                projection,
                visible: expansion_visible,
            },
        );
        cache.available.insert(coord, entities);
    }

    cache.expansion_visible = expansion_visible;
    cache.last_hovered_available = if expansion_visible { hovered } else { None };
    cache.initialized = true;
}

fn set_map_visibility(
    commands: &mut Commands,
    map: &HashMap<StoreChunkCoord, Vec<Entity>>,
    visible: bool,
) {
    let visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for entities in map.values() {
        for entity in entities {
            commands.entity(*entity).insert(visibility);
        }
    }
}

struct RefreshAvailableCoordSpec<'a> {
    cache: &'a mut StoreOverlayCache,
    store: &'a StoreArea,
    world: &'a WorldBounds,
    projection: IsoProjection,
    coord: StoreChunkCoord,
    hovered: bool,
    visible: bool,
}

fn refresh_available_coord(commands: &mut Commands, spec: RefreshAvailableCoordSpec<'_>) {
    if let Some(entities) = spec.cache.available.remove(&spec.coord) {
        for entity in entities {
            commands.entity(entity).despawn();
        }
    }

    let valid =
        validate_chunk_purchase(spec.world, spec.store, spec.coord, StoreChunkKind::Default).valid;
    if !valid {
        return;
    }

    let kind = if spec.hovered {
        StoreChunkOverlayKind::HoveredAvailable
    } else {
        StoreChunkOverlayKind::Available
    };
    let entities = spawn_chunk_outline(
        commands,
        ChunkOutlineSpec {
            store: spec.store,
            coord: spec.coord,
            kind,
            color: if spec.hovered {
                Color::srgba(1.0, 0.86, 0.20, 0.88)
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.15)
            },
            thickness: if spec.hovered { 6.0 } else { 4.0 },
            z: if spec.hovered {
                SortLayer::StoreOverlay.base_z() + 20.0
            } else {
                SortLayer::StoreOverlay.base_z() + 10.0
            },
            projection: spec.projection,
            visible: spec.visible,
        },
    );
    spec.cache.available.insert(spec.coord, entities);
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

struct ChunkOutlineSpec<'a> {
    store: &'a StoreArea,
    coord: StoreChunkCoord,
    kind: StoreChunkOverlayKind,
    color: Color,
    thickness: f32,
    z: f32,
    projection: IsoProjection,
    visible: bool,
}

fn spawn_chunk_outline(commands: &mut Commands, spec: ChunkOutlineSpec<'_>) -> Vec<Entity> {
    let rect = spec.store.chunk_rect(spec.coord);
    let points = [
        rect.min,
        Vec2::new(rect.max.x, rect.min.y),
        rect.max,
        Vec2::new(rect.min.x, rect.max.y),
    ];
    let mut entities = Vec::with_capacity(points.len());

    for (a, b) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let pa = world_to_iso(a, spec.projection);
        let pb = world_to_iso(b, spec.projection);
        let delta = pb - pa;
        let length = delta.length();
        if length <= 0.1 {
            continue;
        }
        let mid = (pa + pb) * 0.5;
        let owner = if spec.kind == StoreChunkOverlayKind::Owned {
            RuntimeOwner::StoreOverlay
        } else {
            RuntimeOwner::ExpansionOverlay
        };

        let entity = commands
            .spawn((
                Sprite::from_color(spec.color, Vec2::new(length, spec.thickness)),
                Transform {
                    translation: Vec3::new(mid.x, mid.y, spec.z),
                    rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                    ..default()
                },
                if spec.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                StoreChunkOverlay {
                    coord: spec.coord,
                    kind: spec.kind,
                },
                StoreChunkOverlaySegment,
                InteractionRole::Overlay,
                RuntimeOwned { owner },
                NonInteractive,
                Name::new(format!(
                    "StoreChunkOverlay {:?} {:?}",
                    spec.kind, spec.coord
                )),
            ))
            .id();
        entities.push(entity);
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_expansion_chunks_respect_policy_and_frontier() {
        let store = StoreArea::new(Vec2::ZERO);
        let world = WorldBounds::default();

        let mut coords = available_expansion_chunks(&world, &store);
        coords.sort_by_key(|coord| (coord.y, coord.x));

        assert_eq!(coords.len(), 9);
        assert!(coords.contains(&StoreChunkCoord { x: -6, y: -4 }));
        assert!(coords.contains(&StoreChunkCoord { x: -6, y: -1 }));
        assert!(coords.contains(&StoreChunkCoord { x: -5, y: -5 }));
        assert!(coords.contains(&StoreChunkCoord { x: -1, y: -5 }));
        assert!(!coords.contains(&StoreChunkCoord { x: -4, y: -4 }));
        assert!(!coords.contains(&StoreChunkCoord { x: -1, y: 0 }));
        assert!(!coords.contains(&StoreChunkCoord { x: 0, y: -1 }));
    }
}
