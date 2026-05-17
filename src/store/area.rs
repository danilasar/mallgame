use bevy::prelude::*;
use std::collections::HashMap;

use crate::presentation::{IsoProjection, world_to_iso};
use crate::store::{
    StoreChunkBounds, StoreChunkCoord, StoreChunkData, StoreChunkKind, StoreExpansionPolicy,
    owned_bounds,
};

#[derive(Resource, Debug, Clone)]
pub struct WorldBounds {
    pub rect: Rect,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            rect: Rect::from_corners(Vec2::new(-1200.0, -1000.0), Vec2::new(1200.0, 1000.0)),
        }
    }
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

    pub fn contains_polygon(&self, polygon: &[Vec2]) -> bool {
        // MVP: vertex-only check.
        // TODO: Strengthen this to check edge intersections or use polygon-vs-union-of-chunks coverage.
        // A large footprint could cross an unowned area even if all vertices are inside owned chunks.
        polygon.iter().all(|point| self.contains_point(*point))
    }

    pub fn owned_chunk_bounds(&self) -> Option<StoreChunkBounds> {
        owned_bounds(&self.owned_chunks)
    }
}

pub struct StorePlugin;

impl Plugin for StorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldBounds>()
            .add_message::<crate::store::PurchaseStoreChunkRequested>()
            .add_systems(Startup, setup_store_area)
            .add_systems(
                Update,
                (
                    crate::store::apply_purchase_store_chunk_requested,
                    clamp_camera_to_world_bounds.after(crate::input::camera_drag_system),
                ),
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
    mut cameras: Query<&mut Transform, With<Camera2d>>,
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

    for mut transform in &mut cameras {
        transform.translation.x = transform.translation.x.clamp(min.x, max.x);
        transform.translation.y = transform.translation.y.clamp(min.y, max.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_store_has_20_chunks() {
        let store = StoreArea::new(Vec2::ZERO);
        assert_eq!(store.owned_chunks.len(), 20);
        assert_eq!(
            store.owned_chunk_bounds().unwrap().min,
            StoreChunkCoord { x: -5, y: -4 }
        );
        assert_eq!(
            store.owned_chunk_bounds().unwrap().max,
            StoreChunkCoord { x: -1, y: -1 }
        );
    }

    #[test]
    fn world_to_chunk_coord_boundaries() {
        let store = StoreArea::new(Vec2::ZERO);
        let size = store.chunk_world_size();

        // Exactly at anchor
        assert_eq!(
            store.world_to_chunk_coord(Vec2::ZERO),
            StoreChunkCoord { x: 0, y: 0 }
        );

        // Slightly left/down from anchor
        assert_eq!(
            store.world_to_chunk_coord(Vec2::new(-0.001, -0.001)),
            StoreChunkCoord { x: -1, y: -1 }
        );

        // Exactly at chunk boundary (positive)
        assert_eq!(
            store.world_to_chunk_coord(size),
            StoreChunkCoord { x: 1, y: 1 }
        );

        // Slightly inside chunk boundary (positive)
        assert_eq!(
            store.world_to_chunk_coord(size - Vec2::splat(0.001)),
            StoreChunkCoord { x: 0, y: 0 }
        );

        // Exactly at chunk boundary (negative)
        assert_eq!(
            store.world_to_chunk_coord(-size),
            StoreChunkCoord { x: -1, y: -1 }
        );

        // Slightly outside chunk boundary (more negative)
        assert_eq!(
            store.world_to_chunk_coord(-size - Vec2::splat(0.001)),
            StoreChunkCoord { x: -2, y: -2 }
        );
    }

    #[test]
    fn contains_point_respects_half_open_interval() {
        let store = StoreArea::new(Vec2::ZERO);
        let size = store.chunk_world_size();

        // Initial store has x: -5..0, y: -4..0
        // Chunk (-1, -1) is [ -size.x, 0 ) x [ -size.y, 0 )

        assert!(store.contains_point(Vec2::new(-0.001, -0.001)));
        assert!(!store.contains_point(Vec2::ZERO)); // (0,0) is chunk (0,0), which is not owned

        assert!(store.contains_point(-size)); // Exact min of (-1,-1)
        assert!(!store.contains_point(Vec2::new(-5.0 * size.x - 0.001, -size.y))); // Just outside the whole store (left)
    }

    #[test]
    fn contains_polygon_requires_owned_union() {
        let store = StoreArea::new(Vec2::ZERO);
        let inside = [
            Vec2::new(-20.0, -20.0),
            Vec2::new(-10.0, -20.0),
            Vec2::new(-10.0, -10.0),
            Vec2::new(-20.0, -10.0),
        ];
        let outside = [
            Vec2::new(10.0, -20.0),
            Vec2::new(20.0, -20.0),
            Vec2::new(20.0, -10.0),
            Vec2::new(10.0, -10.0),
        ];
        assert!(store.contains_polygon(&inside));
        assert!(!store.contains_polygon(&outside));
    }
}
