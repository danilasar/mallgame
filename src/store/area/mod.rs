use bevy::prelude::*;
use std::collections::HashMap;

use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{
    StoreChunkBounds, StoreChunkCoord, StoreChunkData, StoreChunkKind, StoreExpansionPolicy,
    owned_bounds,
};

#[derive(Message, Debug, Clone, Copy)]
pub struct PurchaseStoreChunkRequested {
    pub coord: StoreChunkCoord,
    pub kind: StoreChunkKind,
}

#[derive(Resource, Debug, Clone)]
pub struct WorldBounds {
    pub rect: Rect,
}

impl WorldBounds {
    #[allow(dead_code)]
    pub fn contains_chunk(&self, coord: StoreChunkCoord) -> bool {
        let size = Vec2::splat(128.0);
        let min = Vec2::new(coord.x as f32 * size.x, coord.y as f32 * size.y);
        let chunk_rect = Rect::from_corners(min, min + size);
        self.rect.contains(chunk_rect.min) && self.rect.contains(chunk_rect.max)
    }
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            rect: Rect::from_corners(Vec2::new(-1200.0, -1000.0), Vec2::new(1200.0, 1000.0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoverageSamplingOptions {
    pub max_edge_step: f32,
    pub epsilon: f32,
}

impl Default for CoverageSamplingOptions {
    fn default() -> Self {
        Self {
            max_edge_step: 16.0,
            epsilon: 0.001,
        }
    }
}

pub enum CoverageFailureReason {
    PointOutsideOwnedArea,
    #[allow(dead_code)]
    EmptyPolygon,
}

pub struct CoverageResult {
    pub valid: bool,
    pub failed_point: Option<Vec2>,
    #[allow(dead_code)]
    pub reason: Option<CoverageFailureReason>,
}

#[derive(Resource, Debug, Clone)]
pub struct StoreArea {
    pub anchor: Vec2,
    pub cell_size: Vec2,
    pub chunk_size_cells: UVec2,
    pub owned_chunks: HashMap<StoreChunkCoord, StoreChunkData>,
    pub expansion_policy: StoreExpansionPolicy,
}

impl StoreArea {
    pub fn new(anchor: Vec2) -> Self {
        let mut owned_chunks = HashMap::new();
        for x in -5..0 {
            for y in -4..0 {
                owned_chunks.insert(
                    StoreChunkCoord { x, y },
                    StoreChunkData {
                        kind: StoreChunkKind::Default,
                    },
                );
            }
        }
        Self {
            anchor,
            cell_size: Vec2::splat(32.0),
            chunk_size_cells: UVec2::new(4, 4),
            owned_chunks,
            expansion_policy: StoreExpansionPolicy::default(),
        }
    }

    pub fn chunk_world_size(&self) -> Vec2 {
        self.cell_size * self.chunk_size_cells.as_vec2()
    }

    pub fn chunk_rect(&self, coord: StoreChunkCoord) -> Rect {
        let size = self.chunk_world_size();
        let min = self.anchor + Vec2::new(coord.x as f32 * size.x, coord.y as f32 * size.y);
        Rect::from_corners(min, min + size)
    }

    pub fn world_to_chunk_coord(&self, world_pos: Vec2) -> StoreChunkCoord {
        let size = self.chunk_world_size();
        let local = world_pos - self.anchor;
        StoreChunkCoord {
            x: (local.x / size.x).floor() as i32,
            y: (local.y / size.y).floor() as i32,
        }
    }

    #[allow(dead_code)]
    pub fn contains_point(&self, world_pos: Vec2) -> bool {
        let coord = self.world_to_chunk_coord(world_pos);
        if !self.owned_chunks.contains_key(&coord) {
            return false;
        }
        let rect = self.chunk_rect(coord);
        world_pos.x >= rect.min.x
            && world_pos.x < rect.max.x
            && world_pos.y >= rect.min.y
            && world_pos.y < rect.max.y
    }

    pub fn contains_polygon_sampled(
        &self,
        polygon: &[Vec2],
        options: CoverageSamplingOptions,
    ) -> CoverageResult {
        if polygon.is_empty() {
            return CoverageResult {
                valid: false,
                failed_point: None,
                reason: Some(CoverageFailureReason::EmptyPolygon),
            };
        }

        for (a, b) in polygon
            .iter()
            .copied()
            .zip(polygon.iter().copied().cycle().skip(1))
            .take(polygon.len())
        {
            let delta = b - a;
            let length = delta.length();
            let steps = (length / options.max_edge_step).ceil().max(1.0) as usize;

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let sample = a.lerp(b, t);
                if !self.contains_point_with_epsilon(sample, options.epsilon) {
                    return CoverageResult {
                        valid: false,
                        failed_point: Some(sample),
                        reason: Some(CoverageFailureReason::PointOutsideOwnedArea),
                    };
                }
            }
        }

        CoverageResult {
            valid: true,
            failed_point: None,
            reason: None,
        }
    }

    fn contains_point_with_epsilon(&self, world_pos: Vec2, epsilon: f32) -> bool {
        let coord = self.world_to_chunk_coord(world_pos);
        if !self.owned_chunks.contains_key(&coord) {
            // Check neighbors if close to boundary
            for neighbor in [
                world_pos + Vec2::new(epsilon, 0.0),
                world_pos + Vec2::new(-epsilon, 0.0),
                world_pos + Vec2::new(0.0, epsilon),
                world_pos + Vec2::new(0.0, -epsilon),
            ] {
                let n_coord = self.world_to_chunk_coord(neighbor);
                if self.owned_chunks.contains_key(&n_coord) {
                    return true;
                }
            }
            return false;
        }
        true
    }

    #[allow(dead_code)]
    pub fn owned_chunk_bounds(&self) -> Option<StoreChunkBounds> {
        owned_bounds(&self.owned_chunks)
    }
}

pub struct StorePlugin;

impl Plugin for StorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldBounds>()
            .add_message::<PurchaseStoreChunkRequested>()
            .add_systems(Startup, setup_store_area)
            .add_systems(
                Update,
                clamp_camera_to_world_bounds.after(crate::input::camera_drag_system),
            );
    }
}

fn setup_store_area(mut commands: Commands, world: Res<WorldBounds>) {
    let anchor = (world.rect.min + world.rect.max) * 0.5;
    let store = StoreArea::new(anchor);
    info!("Initial owned store chunks: {}", store.owned_chunks.len());
    commands.insert_resource(store);
}

fn clamp_camera_to_world_bounds(
    world: Res<WorldBounds>,
    projection: Res<IsoProjection>,
    mut camera_query: Query<(&Camera, &Projection, &mut Transform), With<Camera2d>>,
) {
    let corners = [
        world.rect.min,
        Vec2::new(world.rect.max.x, world.rect.min.y),
        world.rect.max,
        Vec2::new(world.rect.min.x, world.rect.max.y),
    ];
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for corner in corners {
        let projected = world_to_iso(corner, *projection);
        min = min.min(projected);
        max = max.max(projected);
    }

    for (camera, projection_component, mut transform) in camera_query.iter_mut() {
        let Some(viewport_size) = camera.logical_viewport_size() else {
            continue;
        };

        let ortho = match projection_component {
            Projection::Orthographic(projection) => projection,
            _ => continue,
        };

        let half_visible = viewport_size * ortho.scale * 0.5;
        let clamp_min = min + half_visible;
        let clamp_max = max - half_visible;

        let center_x = (min.x + max.x) * 0.5;
        let center_y = (min.y + max.y) * 0.5;
        let clamp_x = if clamp_min.x <= clamp_max.x {
            transform.translation.x.clamp(clamp_min.x, clamp_max.x)
        } else {
            center_x
        };
        let clamp_y = if clamp_min.y <= clamp_max.y {
            transform.translation.y.clamp(clamp_min.y, clamp_max.y)
        } else {
            center_y
        };

        transform.translation.x = clamp_x;
        transform.translation.y = clamp_y;
    }
}

#[cfg(test)]
mod tests;
