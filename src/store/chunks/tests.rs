use super::*;

fn data() -> StoreChunkData {
    StoreChunkData {
        kind: StoreChunkKind::Default,
    }
}

#[test]
fn side_neighbors_are_cardinal_only() {
    let n = side_neighbors(StoreChunkCoord { x: 0, y: 0 });
    assert_eq!(n.len(), 4);
    assert!(n.contains(&StoreChunkCoord { x: -1, y: 0 }));
    assert!(n.contains(&StoreChunkCoord { x: 1, y: 0 }));
    assert!(n.contains(&StoreChunkCoord { x: 0, y: -1 }));
    assert!(n.contains(&StoreChunkCoord { x: 0, y: 1 }));
    assert!(!n.contains(&StoreChunkCoord { x: 1, y: 1 }));
}

#[test]
fn detects_simple_enclosed_hole() {
    let mut chunks = HashMap::new();
    // Ring around (0,0)
    for x in -1..=1 {
        for y in -1..=1 {
            if x == 0 && y == 0 {
                continue;
            }
            chunks.insert(StoreChunkCoord { x, y }, data());
        }
    }
    // If we buy something else and (0,0) is still empty, it might be a hole.
    // wait, would_create_hole(chunks, candidate) checks if adding candidate creates a hole.

    // Let's setup a ring with one missing link
    let mut chunks = HashMap::new();
    for coord in [
        StoreChunkCoord { x: -1, y: -1 },
        StoreChunkCoord { x: 0, y: -1 },
        StoreChunkCoord { x: 1, y: -1 },
        StoreChunkCoord { x: -1, y: 0 },
        StoreChunkCoord { x: 1, y: 0 },
        StoreChunkCoord { x: -1, y: 1 },
        StoreChunkCoord { x: 0, y: 1 },
        // (1,1) is missing
    ] {
        chunks.insert(coord, data());
    }
    // Adding (1,1) closes the ring, leaving (0,0) as a hole.
    assert!(would_create_hole(&chunks, StoreChunkCoord { x: 1, y: 1 }));
}

#[test]
fn adding_to_solid_block_does_not_create_hole() {
    let mut chunks = HashMap::new();
    chunks.insert(StoreChunkCoord { x: 0, y: 0 }, data());
    assert!(!would_create_hole(&chunks, StoreChunkCoord { x: 1, y: 0 }));
}

#[test]
fn diagonal_purchase_without_filling_middle_is_not_hole_per_se_but_flood_fill_checks_enclosed_empty_space()
 {
    let mut chunks = HashMap::new();
    chunks.insert(StoreChunkCoord { x: 0, y: 0 }, data());
    // (1,1) is diagonal. Adding it doesn't enclose any space.
    assert!(!would_create_hole(&chunks, StoreChunkCoord { x: 1, y: 1 }));
}

#[test]
fn large_solid_block_no_hole() {
    let mut chunks = HashMap::new();
    for x in 0..5 {
        for y in 0..5 {
            chunks.insert(StoreChunkCoord { x, y }, data());
        }
    }
    assert!(!would_create_hole(&chunks, StoreChunkCoord { x: 5, y: 0 }));
}
