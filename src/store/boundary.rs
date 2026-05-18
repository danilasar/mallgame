use bevy::ecs::system::SystemParam;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::mesh::Indices;
use std::collections::{HashMap, HashSet};

use crate::objects::components::{
    Interactive, InteractionRole, RuntimeOwned, RuntimeOwner, SortLayer,
};
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{
    StoreArea, StoreChunkCoord, StoreExpansionPolicy, WorldBounds,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum StoreBoundarySide {
    Top = 0,
    Right = 1,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct WallSegmentKey {
    pub chunk: StoreChunkCoord,
    pub side: StoreBoundarySide,
}

#[derive(Clone, Copy, Debug)]
pub struct StoreBoundarySegment {
    pub key: WallSegmentKey,
    pub start: Vec2,
    pub end: Vec2,
    pub normal: Vec2,
    pub length: f32,
    pub height: f32,
    pub thickness: f32,
}

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct StoreWallSegment {
    pub key: WallSegmentKey,
}

#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct WallSurface {
    pub key: WallSegmentKey,
    pub start: Vec2,
    pub end: Vec2,
    pub length: f32,
    pub height: f32,
    pub thickness: f32,
    pub normal: Vec2,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WallVisual;

#[derive(Resource, Debug, Default)]
pub struct WallVisualCache {
    pub entities_by_key: HashMap<WallSegmentKey, Entity>,
    pub initialized: bool,
}

pub struct StoreBoundaryPlugin;

impl Plugin for StoreBoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WallVisualCache>()
            .add_systems(Update, sync_store_boundaries.in_set(crate::store::commands::DomainCommandSet::PostDomainApply));
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
struct StoreBoundaryParams<'w, 's> {
    commands: Commands<'w, 's>,
    store: Res<'w, StoreArea>,
    world: Res<'w, WorldBounds>,
    projection: Res<'w, IsoProjection>,
    cache: ResMut<'w, WallVisualCache>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

fn sync_store_boundaries(mut params: StoreBoundaryParams) {
    if !params.cache.initialized || params.store.is_changed() || params.world.is_changed() {
        let expected = collect_boundary_segments(&params.store, &params.world);
        sync_wall_cache(
            &mut params.commands,
            &mut params.cache,
            &mut params.meshes,
            &mut params.materials,
            expected,
            *params.projection,
        );
        params.cache.initialized = true;
    }
}

pub fn clear_wall_cache(cache: &mut WallVisualCache) {
    cache.entities_by_key.clear();
    cache.initialized = false;
}

pub fn collect_boundary_segments(store: &StoreArea, world: &WorldBounds) -> Vec<StoreBoundarySegment> {
    let mut segments = Vec::new();
    segments.extend(boundary_line_segments(store, world, StoreBoundarySide::Top));
    segments.extend(boundary_line_segments(store, world, StoreBoundarySide::Right));
    segments.sort_by_key(|segment| boundary_sort_key(segment.key));
    segments
}

pub fn is_locked_boundary_side(policy: StoreExpansionPolicy, side: StoreBoundarySide) -> bool {
    match side {
        StoreBoundarySide::Top => !policy.allow_up,
        StoreBoundarySide::Right => !policy.allow_right,
    }
}

fn boundary_sort_key(key: WallSegmentKey) -> (i32, i32, u8) {
    (key.chunk.y, key.chunk.x, key.side as u8)
}

fn boundary_line_segments(
    store: &StoreArea,
    world: &WorldBounds,
    side: StoreBoundarySide,
) -> Vec<StoreBoundarySegment> {
    if !is_locked_boundary_side(store.expansion_policy, side) {
        return Vec::new();
    }

    let Some(bounds) = store.owned_chunk_bounds() else {
        return Vec::new();
    };
    let chunk_size = store.chunk_world_size();
    let wall_height = chunk_size.y * 1.5;
    let wall_thickness = 8.0;

    let mut segments = Vec::new();

    match side {
        StoreBoundarySide::Top => {
            let y = bounds.max.y;
            for x in (bounds.min.x..=bounds.max.x).rev() {
                let coord = StoreChunkCoord { x, y };
                let Some(_) = store.owned_chunks.get(&coord) else {
                    break;
                };

                let rect = store.chunk_rect(coord);
                let start = Vec2::new(rect.min.x, rect.max.y);
                let end = Vec2::new(rect.max.x, rect.max.y);
                if !world.rect.contains(start) || !world.rect.contains(end) {
                    break;
                }

                segments.push(StoreBoundarySegment {
                    key: WallSegmentKey { chunk: coord, side },
                    start,
                    end,
                    normal: Vec2::Y,
                    length: start.distance(end),
                    height: wall_height,
                    thickness: wall_thickness,
                });
            }
        }
        StoreBoundarySide::Right => {
            let x = bounds.max.x;
            for y in (bounds.min.y..=bounds.max.y).rev() {
                let coord = StoreChunkCoord { x, y };
                let Some(_) = store.owned_chunks.get(&coord) else {
                    break;
                };

                let rect = store.chunk_rect(coord);
                let start = Vec2::new(rect.max.x, rect.min.y);
                let end = Vec2::new(rect.max.x, rect.max.y);
                if !world.rect.contains(start) || !world.rect.contains(end) {
                    break;
                }

                segments.push(StoreBoundarySegment {
                    key: WallSegmentKey { chunk: coord, side },
                    start,
                    end,
                    normal: Vec2::X,
                    length: start.distance(end),
                    height: wall_height,
                    thickness: wall_thickness,
                });
            }
        }
    }

    segments
}

fn sync_wall_cache(
    commands: &mut Commands,
    cache: &mut WallVisualCache,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    expected: Vec<StoreBoundarySegment>,
    projection: IsoProjection,
) {
    let expected_keys: HashSet<_> = expected.iter().map(|segment| segment.key).collect();

    let stale_keys: Vec<_> = cache
        .entities_by_key
        .keys()
        .copied()
        .filter(|key| !expected_keys.contains(key))
        .collect();

    for key in stale_keys {
        if let Some(entity) = cache.entities_by_key.remove(&key) {
            commands.entity(entity).despawn();
        }
    }

    for segment in expected {
        if cache.entities_by_key.contains_key(&segment.key) {
            continue;
        }

        let entity = spawn_wall_segment(commands, meshes, materials, segment, projection);
        cache.entities_by_key.insert(segment.key, entity);
    }
}

fn spawn_wall_segment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    segment: StoreBoundarySegment,
    projection: IsoProjection,
) -> Entity {
    let projected_start = world_to_iso(segment.start, projection);
    let projected_end = world_to_iso(segment.end, projection);
    let wall_direction = projected_end - projected_start;
    let wall_normal = if wall_direction.length_squared() > f32::EPSILON {
        Vec2::new(-wall_direction.y, wall_direction.x).normalize()
    } else {
        Vec2::Y
    };
    let thickness_offset = wall_normal * segment.thickness;

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let vertices = vec![
        [projected_start.x, projected_start.y, 0.0],
        [projected_end.x, projected_end.y, 0.0],
        [
            projected_end.x + thickness_offset.x,
            projected_end.y + thickness_offset.y + segment.height,
            0.0,
        ],
        [
            projected_start.x + thickness_offset.x,
            projected_start.y + thickness_offset.y + segment.height,
            0.0,
        ],
    ];

    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    let color = match segment.key.side {
        StoreBoundarySide::Top => Color::srgba(0.35, 0.25, 0.18, 0.98),
        StoreBoundarySide::Right => Color::srgba(0.28, 0.18, 0.12, 0.98),
    };

    commands
        .spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_xyz(0.0, 0.0, SortLayer::WallFace.base_z()),
            Visibility::Visible,
            StoreWallSegment { key: segment.key },
            WallSurface {
                key: segment.key,
                start: segment.start,
                end: segment.end,
                length: segment.length,
                height: segment.height,
                thickness: segment.thickness,
                normal: segment.normal,
            },
            WallVisual,
            InteractionRole::WallSurface,
            RuntimeOwned {
                owner: RuntimeOwner::BoundaryWall,
            },
            Interactive,
            Name::new(format!(
                "StoreWallSegment {:?} {:?}",
                segment.key.side, segment.key.chunk
            )),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_store_generates_two_corner_segments() {
        let store = StoreArea::new(Vec2::ZERO);
        let world = WorldBounds::default();
        let segments = collect_boundary_segments(&store, &world);

        assert_eq!(segments.len(), 9);
        assert!(segments.iter().any(|segment| segment.key.side == StoreBoundarySide::Top));
        assert!(segments.iter().any(|segment| segment.key.side == StoreBoundarySide::Right));

        let chunk_size = store.chunk_world_size();
        for segment in segments {
            assert!((segment.height - chunk_size.y * 1.5).abs() < f32::EPSILON);
            assert!((segment.length - chunk_size.x).abs() < f32::EPSILON);
            assert!((segment.thickness - 8.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn missing_outer_corner_does_not_shift_wall_inward() {
        let mut store = StoreArea::new(Vec2::ZERO);
        store.owned_chunks.remove(&StoreChunkCoord { x: -1, y: -1 });

        let world = WorldBounds::default();
        let segments = collect_boundary_segments(&store, &world);

        assert!(segments.is_empty());
    }

    #[test]
    fn top_row_stops_at_first_gap_from_corner() {
        let mut store = StoreArea::new(Vec2::ZERO);
        store.owned_chunks.remove(&StoreChunkCoord { x: -2, y: -1 });

        let world = WorldBounds::default();
        let segments = collect_boundary_segments(&store, &world);

        let top_segments = segments
            .iter()
            .filter(|segment| segment.key.side == StoreBoundarySide::Top)
            .count();
        let right_segments = segments
            .iter()
            .filter(|segment| segment.key.side == StoreBoundarySide::Right)
            .count();

        assert_eq!(top_segments, 1);
        assert_eq!(right_segments, 4);
    }
}
