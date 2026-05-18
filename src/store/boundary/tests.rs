use super::*;

#[test]
fn initial_store_generates_boundary_runs() {
    let store = StoreArea::new(Vec2::ZERO);
    let world = WorldBounds::default();
    let segments = collect_boundary_segments(&store, &world);

    assert_eq!(segments.len(), 9);
    assert!(
        segments
            .iter()
            .any(|segment| segment.key.side == StoreBoundarySide::Top)
    );
    assert!(
        segments
            .iter()
            .any(|segment| segment.key.side == StoreBoundarySide::Right)
    );

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

#[test]
fn test_boundary_wall_interior_direction() {
    assert_eq!(
        boundary_wall_interior_direction(StoreBoundarySide::Top),
        Vec2::NEG_Y
    );
    assert_eq!(
        boundary_wall_interior_direction(StoreBoundarySide::Right),
        Vec2::NEG_X
    );
}
