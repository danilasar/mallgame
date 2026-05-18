#[cfg(test)]
mod tests;
pub mod opening;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::mesh::Indices;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::objects::components::{
    AccessZoneReason, DerivedDoorPlacement, InteractionRole, Interactive, InteriorAccessZone,
    RuntimeOwned, RuntimeOwner, SortLayer, VisualOffset, WallAttachmentPoint, WallMounted,
    WallOccupancyKind, WallOpeningComponent, WorldPos, derive_wallprint,
};
use crate::tools::NonInteractive;
use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{StoreArea, StoreChunkCoord, StoreExpansionPolicy, WorldBounds};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreBoundarySide {
    Top = 0,
    Right = 1,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

/// Marker for a rendered wall visual piece (Mesh2d, NonInteractive).
/// One or more per wall segment depending on openings.
#[derive(Component, Debug, Clone, Copy)]
pub struct WallVisualPiece {
    #[allow(dead_code)]
    pub key: WallSegmentKey,
}

/// Kind of a wall visual piece for future material/z differentiation.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallPieceKind {
    Solid,
    Glass,
    #[allow(dead_code)]
    Frame,
}

/// Dirty set: segments that need their visual pieces rebuilt this frame.
#[derive(Resource, Debug, Default)]
pub struct DirtyWallOpeningSegments {
    pub dirty: HashSet<WallSegmentKey>,
}

#[derive(Resource, Debug, Default)]
pub struct WallVisualCache {
    /// Surface entity per segment (WallSurface + Interactive, no Mesh2d).
    pub surface_by_key: HashMap<WallSegmentKey, Entity>,
    /// Visual piece entities per segment (Mesh2d + NonInteractive).
    pub pieces_by_key: HashMap<WallSegmentKey, Vec<Entity>>,
    pub initialized: bool,
}

pub struct StoreBoundaryPlugin;

impl Plugin for StoreBoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WallVisualCache>()
            .init_resource::<DirtyWallOpeningSegments>()
            .add_systems(
                Update,
                (
                    sync_store_boundaries,
                    rebuild_dirty_wall_visuals,
                    sync_wall_mounted_object_positions,
                )
                    .chain()
                    .in_set(crate::store::commands::DomainCommandSet::PostDomainApply),
            );
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
struct StoreBoundaryParams<'w, 's> {
    commands: Commands<'w, 's>,
    store: Res<'w, StoreArea>,
    world: Res<'w, WorldBounds>,
    cache: ResMut<'w, WallVisualCache>,
    dirty: ResMut<'w, DirtyWallOpeningSegments>,
}

pub fn wall_surface_visual_offset(
    surface: &WallSurface,
    projection: IsoProjection,
    height_on_wall: f32,
) -> Vec2 {
    let projected_start = world_to_iso(surface.start, projection);
    let projected_end = world_to_iso(surface.end, projection);
    let segment = projected_end - projected_start;
    if segment.length_squared() <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let wall_direction = segment.normalize();
    let wall_normal = Vec2::new(-wall_direction.y, wall_direction.x);
    wall_normal * surface.thickness + Vec2::new(0.0, height_on_wall.clamp(0.0, surface.height))
}

pub fn wall_surface_world_pos(surface: &WallSurface, offset_along_segment: f32) -> Vec2 {
    let t = (offset_along_segment / surface.length.max(f32::EPSILON)).clamp(0.0, 1.0);
    surface.start.lerp(surface.end, t)
}

fn sync_wall_mounted_object_positions(
    projection: Res<IsoProjection>,
    surfaces: Query<&WallSurface>,
    mut mounted: Query<(&WallMounted, &mut WorldPos, &mut VisualOffset)>,
) {
    for (mounted, mut world_pos, mut visual_offset) in &mut mounted {
        let Some(surface) = surfaces
            .iter()
            .find(|surface| surface.key == mounted.attachment.segment_key)
        else {
            continue;
        };

        world_pos.0 = wall_surface_world_pos(surface, mounted.attachment.offset_along_segment);
        visual_offset.0 =
            wall_surface_visual_offset(surface, *projection, mounted.attachment.height_on_wall);
    }
}

fn sync_store_boundaries(mut params: StoreBoundaryParams) {
    if !params.cache.initialized || params.store.is_changed() || params.world.is_changed() {
        let expected = collect_boundary_segments(&params.store, &params.world);
        sync_wall_surface_cache(
            &mut params.commands,
            &mut params.cache,
            &mut params.dirty,
            expected,
        );
        params.cache.initialized = true;
    }
}

/// Clear the wall cache on world reset. Entities are despawned via RuntimeOwned cleanup elsewhere.
pub fn clear_wall_cache(cache: &mut WallVisualCache) {
    cache.surface_by_key.clear();
    cache.pieces_by_key.clear();
    cache.initialized = false;
}

pub fn collect_boundary_segments(
    store: &StoreArea,
    world: &WorldBounds,
) -> Vec<StoreBoundarySegment> {
    let mut segments = Vec::new();
    segments.extend(boundary_line_segments(store, world, StoreBoundarySide::Top));
    segments.extend(boundary_line_segments(
        store,
        world,
        StoreBoundarySide::Right,
    ));
    segments.sort_by_key(|segment| boundary_sort_key(segment.key));
    segments
}

pub fn boundary_wall_interior_direction(side: StoreBoundarySide) -> Vec2 {
    match side {
        StoreBoundarySide::Top => Vec2::NEG_Y,
        StoreBoundarySide::Right => Vec2::NEG_X,
    }
}

pub fn derive_door_placement(
    wall_width: f32,
    wall_height: f32,
    access_width: f32,
    access_depth: f32,
    attachment: WallAttachmentPoint,
    surface: &WallSurface,
    occupancy_kind: WallOccupancyKind,
) -> Result<DerivedDoorPlacement, crate::store::PlacementInvalidReason> {
    let wallprint = derive_wallprint(attachment, wall_width, wall_height, occupancy_kind);

    let wall_dir = (surface.end - surface.start).normalize();
    let interior_dir = boundary_wall_interior_direction(surface.key.side);

    let base_pos = surface.start + wall_dir * attachment.offset_along_segment;

    // Half width along the wall
    let half_width = access_width * 0.5;

    // The points of the interior access zone polygon
    // p1, p2 are along the wall base
    let p1 = base_pos - wall_dir * half_width;
    let p2 = base_pos + wall_dir * half_width;
    // p3, p4 are projected into the interior
    let p3 = p2 + interior_dir * access_depth;
    let p4 = p1 + interior_dir * access_depth;

    Ok(DerivedDoorPlacement {
        wallprint,
        interior_access_zone: InteriorAccessZone {
            polygon: vec![p1, p2, p3, p4],
            reason: AccessZoneReason::DoorAccess,
        },
    })
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
                    continue;
                };

                let rect = store.chunk_rect(coord);
                let start = Vec2::new(rect.min.x, rect.max.y);
                let end = Vec2::new(rect.max.x, rect.max.y);
                if !world.rect.contains(start) || !world.rect.contains(end) {
                    continue;
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
                    continue;
                };

                let rect = store.chunk_rect(coord);
                let start = Vec2::new(rect.max.x, rect.min.y);
                let end = Vec2::new(rect.max.x, rect.max.y);
                if !world.rect.contains(start) || !world.rect.contains(end) {
                    continue;
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

fn sync_wall_surface_cache(
    commands: &mut Commands,
    cache: &mut WallVisualCache,
    dirty: &mut DirtyWallOpeningSegments,
    expected: Vec<StoreBoundarySegment>,
) {
    let expected_keys: HashSet<_> = expected.iter().map(|segment| segment.key).collect();

    let stale_keys: Vec<_> = cache
        .surface_by_key
        .keys()
        .copied()
        .filter(|key| !expected_keys.contains(key))
        .collect();

    for key in stale_keys {
        if let Some(entity) = cache.surface_by_key.remove(&key) {
            commands.entity(entity).try_despawn();
        }
        // Despawn stale visual pieces; dirty marking not needed (segment gone)
        for piece in cache.pieces_by_key.remove(&key).unwrap_or_default() {
            commands.entity(piece).try_despawn();
        }
    }

    for segment in expected {
        if cache.surface_by_key.contains_key(&segment.key) {
            continue;
        }

        let entity = spawn_wall_surface_entity(commands, segment);
        cache.surface_by_key.insert(segment.key, entity);
        // Mark dirty so rebuild_dirty_wall_visuals spawns the initial visual pieces
        dirty.dirty.insert(segment.key);
    }
}

/// Spawn the logical wall surface entity: WallSurface + picking, NO Mesh2d.
fn spawn_wall_surface_entity(commands: &mut Commands, segment: StoreBoundarySegment) -> Entity {
    commands
        .spawn((
            Transform::default(),
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

/// Build a Mesh for a rectangular wall piece (sub-rect of a wall segment).
fn build_wall_piece_mesh(
    piece: &opening::WallPieceRect,
    segment: &StoreBoundarySegment,
    projection: IsoProjection,
) -> Mesh {
    let wall_dir_world = if segment.length > f32::EPSILON {
        (segment.end - segment.start) / segment.length
    } else {
        Vec2::X
    };

    let piece_start_world = segment.start + wall_dir_world * piece.offset_min;
    let piece_end_world = segment.start + wall_dir_world * piece.offset_max;

    let projected_start = world_to_iso(piece_start_world, projection);
    let projected_end = world_to_iso(piece_end_world, projection);

    let wall_dir_proj = projected_end - projected_start;
    let wall_normal_proj = if wall_dir_proj.length_squared() > f32::EPSILON {
        Vec2::new(-wall_dir_proj.y, wall_dir_proj.x).normalize()
    } else {
        Vec2::Y
    };
    let thickness_offset = wall_normal_proj * segment.thickness;

    // Thickness is interpolated linearly from 0 at h=0 to full at h=surface_height.
    // Applying a proportional fraction to bottom vertices ensures adjacent pieces
    // (e.g. a full-height strip and a piece above an opening) share exact seam corners.
    let h_inv = 1.0 / segment.height.max(f32::EPSILON);
    let thick_min = thickness_offset * (piece.height_min * h_inv);
    let thick_max = thickness_offset * (piece.height_max * h_inv);

    // Four corners: bottom-left, bottom-right, top-right, top-left
    let bl = projected_start + thick_min + Vec2::new(0.0, piece.height_min);
    let br = projected_end + thick_min + Vec2::new(0.0, piece.height_min);
    let tr = projected_end + thick_max + Vec2::new(0.0, piece.height_max);
    let tl = projected_start + thick_max + Vec2::new(0.0, piece.height_max);

    // UV: u along segment width, v from top (0) to bottom (1) to match original
    let u_min = piece.offset_min / segment.length.max(f32::EPSILON);
    let u_max = piece.offset_max / segment.length.max(f32::EPSILON);
    let v_min = 1.0 - piece.height_max / segment.height.max(f32::EPSILON);
    let v_max = 1.0 - piece.height_min / segment.height.max(f32::EPSILON);

    let vertices = vec![
        [bl.x, bl.y, 0.0],
        [br.x, br.y, 0.0],
        [tr.x, tr.y, 0.0],
        [tl.x, tl.y, 0.0],
    ];
    let uvs = vec![[u_min, v_max], [u_max, v_max], [u_max, v_min], [u_min, v_min]];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn right_neighbor_key(key: WallSegmentKey) -> WallSegmentKey {
    match key.side {
        StoreBoundarySide::Top => WallSegmentKey {
            chunk: StoreChunkCoord { x: key.chunk.x + 1, y: key.chunk.y },
            side: key.side,
        },
        StoreBoundarySide::Right => WallSegmentKey {
            chunk: StoreChunkCoord { x: key.chunk.x, y: key.chunk.y + 1 },
            side: key.side,
        },
    }
}

fn left_neighbor_key(key: WallSegmentKey) -> WallSegmentKey {
    match key.side {
        StoreBoundarySide::Top => WallSegmentKey {
            chunk: StoreChunkCoord { x: key.chunk.x - 1, y: key.chunk.y },
            side: key.side,
        },
        StoreBoundarySide::Right => WallSegmentKey {
            chunk: StoreChunkCoord { x: key.chunk.x, y: key.chunk.y - 1 },
            side: key.side,
        },
    }
}

/// System: rebuild visual piece entities for all dirty wall segments.
#[allow(clippy::too_many_arguments)]
fn rebuild_dirty_wall_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    projection: Res<IsoProjection>,
    mut cache: ResMut<WallVisualCache>,
    mut dirty: ResMut<DirtyWallOpeningSegments>,
    surfaces: Query<&WallSurface>,
    openings: Query<&WallOpeningComponent>,
    store: Res<StoreArea>,
    world: Res<WorldBounds>,
) {
    if dirty.dirty.is_empty() {
        return;
    }

    let all_segments = collect_boundary_segments(&store, &world);
    let seg_len_by_key: HashMap<WallSegmentKey, f32> =
        all_segments.iter().map(|s| (s.key, s.length)).collect();

    let initial_dirty: HashSet<WallSegmentKey> = dirty.dirty.drain().collect();

    // Expand dirty set: always include immediate neighbors so cross-segment cutouts
    // are correctly cleared and rebuilt when an opening moves between segments.
    let mut expanded: HashSet<WallSegmentKey> = initial_dirty.clone();
    for &key in &initial_dirty {
        let left = left_neighbor_key(key);
        let right = right_neighbor_key(key);
        if seg_len_by_key.contains_key(&left) {
            expanded.insert(left);
        }
        if seg_len_by_key.contains_key(&right) {
            expanded.insert(right);
        }
    }

    for key in expanded {
        // Despawn stale visual pieces for this segment
        let stale = cache.pieces_by_key.remove(&key).unwrap_or_default();
        for e in stale {
            commands.entity(e).try_despawn();
        }

        let surface = surfaces.iter().find(|s| s.key == key);
        let segment = all_segments.iter().find(|s| s.key == key);
        let (Some(surface), Some(segment)) = (surface, segment) else {
            continue;
        };

        let seg_len = surface.length;
        let left_key = left_neighbor_key(key);
        let left_seg_len = seg_len_by_key.get(&left_key).copied().unwrap_or(0.0);
        let right_key = right_neighbor_key(key);

        // Collect openings affecting this segment from three sources:
        // 1. Direct: opening's primary segment is this key (clipped to [0, seg_len])
        // 2. Left-neighbor right-overflow: opening in left_key overflows past left_seg_len
        // 3. Right-neighbor left-underflow: opening in right_key has offset_min < 0
        let mut seg_openings: Vec<opening::WallOpeningRect> = Vec::new();

        for o in openings.iter().filter(|o| o.segment_key == key) {
            let cmin = o.offset_min.max(0.0);
            let cmax = o.offset_max.min(seg_len);
            if cmin < cmax {
                seg_openings.push(opening::WallOpeningRect {
                    offset_min: cmin,
                    offset_max: cmax,
                    height_min: o.height_min,
                    height_max: o.height_max,
                });
            }
        }
        for o in openings.iter().filter(|o| o.segment_key == left_key && o.offset_max > left_seg_len) {
            let overflow = (o.offset_max - left_seg_len).min(seg_len);
            if overflow > 0.0 {
                seg_openings.push(opening::WallOpeningRect {
                    offset_min: 0.0,
                    offset_max: overflow,
                    height_min: o.height_min,
                    height_max: o.height_max,
                });
            }
        }
        for o in openings.iter().filter(|o| o.segment_key == right_key && o.offset_min < 0.0) {
            let left_extent = (-o.offset_min).min(seg_len);
            if left_extent > 0.0 {
                seg_openings.push(opening::WallOpeningRect {
                    offset_min: seg_len - left_extent,
                    offset_max: seg_len,
                    height_min: o.height_min,
                    height_max: o.height_max,
                });
            }
        }

        let wall_color = match key.side {
            StoreBoundarySide::Top => Color::srgba(0.35, 0.25, 0.18, 0.98),
            StoreBoundarySide::Right => Color::srgba(0.28, 0.18, 0.12, 0.98),
        };

        let pieces =
            opening::split_wall_around_openings(surface.length, surface.height, &seg_openings);

        let mut new_entities: Vec<Entity> = Vec::new();

        for piece in &pieces {
            let mesh = build_wall_piece_mesh(piece, segment, *projection);
            let e = commands
                .spawn((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(materials.add(ColorMaterial::from(wall_color))),
                    Transform::from_xyz(0.0, 0.0, SortLayer::WallFace.base_z()),
                    Visibility::Visible,
                    WallVisualPiece { key },
                    WallPieceKind::Solid,
                    NonInteractive,
                    RuntimeOwned { owner: RuntimeOwner::BoundaryWall },
                    Name::new(format!("WallPiece {:?} {:?}", key.side, key.chunk)),
                ))
                .id();
            new_entities.push(e);
        }

        // Glass overlays — same three-source logic, but preserve original offset coords
        // for direct openings and compute correct coords for overflow cases.
        let spawn_glass = |commands: &mut Commands,
                           meshes: &mut Assets<Mesh>,
                           materials: &mut Assets<ColorMaterial>,
                           piece: opening::WallPieceRect,
                           glass_color: Color,
                           segment: &StoreBoundarySegment,
                           projection: IsoProjection,
                           key: WallSegmentKey|
         -> Entity {
            let mesh = build_wall_piece_mesh(&piece, segment, projection);
            commands
                .spawn((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(materials.add(ColorMaterial::from(glass_color))),
                    Transform::from_xyz(0.0, 0.0, SortLayer::WallFace.base_z() + 2.0),
                    Visibility::Visible,
                    WallVisualPiece { key },
                    WallPieceKind::Glass,
                    NonInteractive,
                    RuntimeOwned { owner: RuntimeOwner::BoundaryWall },
                    Name::new(format!("WallGlass {:?} {:?}", key.side, key.chunk)),
                ))
                .id()
        };

        for o in openings.iter().filter(|o| o.segment_key == key) {
            let Some(glass_color) = o.glass_color else { continue };
            let cmin = o.offset_min.max(0.0);
            let cmax = o.offset_max.min(seg_len);
            if cmin < cmax {
                let e = spawn_glass(
                    &mut commands, &mut meshes, &mut materials,
                    opening::WallPieceRect { offset_min: cmin, offset_max: cmax, height_min: o.height_min, height_max: o.height_max },
                    glass_color, segment, *projection, key,
                );
                new_entities.push(e);
            }
        }
        for o in openings.iter().filter(|o| o.segment_key == left_key && o.offset_max > left_seg_len) {
            let Some(glass_color) = o.glass_color else { continue };
            let overflow = (o.offset_max - left_seg_len).min(seg_len);
            if overflow > 0.0 {
                let e = spawn_glass(
                    &mut commands, &mut meshes, &mut materials,
                    opening::WallPieceRect { offset_min: 0.0, offset_max: overflow, height_min: o.height_min, height_max: o.height_max },
                    glass_color, segment, *projection, key,
                );
                new_entities.push(e);
            }
        }
        for o in openings.iter().filter(|o| o.segment_key == right_key && o.offset_min < 0.0) {
            let Some(glass_color) = o.glass_color else { continue };
            let left_extent = (-o.offset_min).min(seg_len);
            if left_extent > 0.0 {
                let e = spawn_glass(
                    &mut commands, &mut meshes, &mut materials,
                    opening::WallPieceRect { offset_min: seg_len - left_extent, offset_max: seg_len, height_min: o.height_min, height_max: o.height_max },
                    glass_color, segment, *projection, key,
                );
                new_entities.push(e);
            }
        }

        cache.pieces_by_key.insert(key, new_entities);
    }
}
