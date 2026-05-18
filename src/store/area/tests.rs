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
fn contains_polygon_sampled_rejects_edge_crossing() {
    let store = StoreArea::new(Vec2::ZERO);

    // Owned chunks are x: -5..-1, y: -4..-1
    // Let's test a diagonal that crosses outside.
    // Owned box: [-5*128, -4*128] to [0, 0] approx (anchor at zero).
    let p1 = Vec2::new(-10.0, -10.0); // Inside
    let p2 = Vec2::new(10.0, 10.0); // Outside

    let poly = [p1, p2]; // Degenerate but works for edge sampling
    let result = store.contains_polygon_sampled(&poly, CoverageSamplingOptions::default());
    assert!(!result.valid);
    assert!(result.failed_point.is_some());
}
