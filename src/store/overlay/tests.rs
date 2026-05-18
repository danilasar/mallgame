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
