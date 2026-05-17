use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StoreChunkCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreChunkKind {
    Default,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StoreChunkData {
    pub kind: StoreChunkKind,
}

#[derive(Debug, Clone, Copy)]
pub struct StoreExpansionPolicy {
    pub allow_left: bool,
    pub allow_right: bool,
    pub allow_up: bool,
    pub allow_down: bool,
    pub require_side_adjacency: bool,
    pub forbid_holes: bool,
}

impl Default for StoreExpansionPolicy {
    fn default() -> Self {
        Self {
            allow_left: true,
            allow_right: false,
            allow_up: false,
            allow_down: true,
            require_side_adjacency: true,
            forbid_holes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreChunkBounds {
    pub min: StoreChunkCoord,
    pub max: StoreChunkCoord,
}

pub fn side_neighbors(coord: StoreChunkCoord) -> [StoreChunkCoord; 4] {
    [
        StoreChunkCoord {
            x: coord.x - 1,
            y: coord.y,
        },
        StoreChunkCoord {
            x: coord.x + 1,
            y: coord.y,
        },
        StoreChunkCoord {
            x: coord.x,
            y: coord.y - 1,
        },
        StoreChunkCoord {
            x: coord.x,
            y: coord.y + 1,
        },
    ]
}

pub fn owned_bounds(chunks: &HashMap<StoreChunkCoord, StoreChunkData>) -> Option<StoreChunkBounds> {
    let mut iter = chunks.keys().copied();
    let first = iter.next()?;
    let mut min = first;
    let mut max = first;
    for coord in iter {
        min.x = min.x.min(coord.x);
        min.y = min.y.min(coord.y);
        max.x = max.x.max(coord.x);
        max.y = max.y.max(coord.y);
    }
    Some(StoreChunkBounds { min, max })
}

pub fn would_create_hole(
    owned_chunks: &HashMap<StoreChunkCoord, StoreChunkData>,
    candidate: StoreChunkCoord,
) -> bool {
    let mut occupied: HashSet<StoreChunkCoord> = owned_chunks.keys().copied().collect();
    occupied.insert(candidate);

    let Some(bounds) = occupied_bounds(&occupied) else {
        return false;
    };

    let min_x = bounds.min.x - 1;
    let max_x = bounds.max.x + 1;
    let min_y = bounds.min.y - 1;
    let max_y = bounds.max.y + 1;
    let start = StoreChunkCoord { x: min_x, y: min_y };
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([start]);
    seen.insert(start);

    while let Some(coord) = queue.pop_front() {
        for next in side_neighbors(coord) {
            if next.x < min_x
                || next.x > max_x
                || next.y < min_y
                || next.y > max_y
                || occupied.contains(&next)
                || seen.contains(&next)
            {
                continue;
            }
            seen.insert(next);
            queue.push_back(next);
        }
    }

    for x in (min_x + 1)..max_x {
        for y in (min_y + 1)..max_y {
            let coord = StoreChunkCoord { x, y };
            if !occupied.contains(&coord) && !seen.contains(&coord) {
                return true;
            }
        }
    }

    false
}

fn occupied_bounds(occupied: &HashSet<StoreChunkCoord>) -> Option<StoreChunkBounds> {
    let mut iter = occupied.iter().copied();
    let first = iter.next()?;
    let mut min = first;
    let mut max = first;
    for coord in iter {
        min.x = min.x.min(coord.x);
        min.y = min.y.min(coord.y);
        max.x = max.x.max(coord.x);
        max.y = max.y.max(coord.y);
    }
    Some(StoreChunkBounds { min, max })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> StoreChunkData {
        StoreChunkData {
            kind: StoreChunkKind::Default,
        }
    }

    #[test]
    fn side_neighbors_are_cardinal_only() {
        let n = side_neighbors(StoreChunkCoord { x: 0, y: 0 });
        assert!(n.contains(&StoreChunkCoord { x: -1, y: 0 }));
        assert!(n.contains(&StoreChunkCoord { x: 1, y: 0 }));
        assert!(n.contains(&StoreChunkCoord { x: 0, y: -1 }));
        assert!(n.contains(&StoreChunkCoord { x: 0, y: 1 }));
        assert!(!n.contains(&StoreChunkCoord { x: 1, y: 1 }));
    }

    #[test]
    fn detects_simple_enclosed_hole() {
        let mut chunks = HashMap::new();
        for coord in [
            StoreChunkCoord { x: -1, y: -1 },
            StoreChunkCoord { x: 0, y: -1 },
            StoreChunkCoord { x: 1, y: -1 },
            StoreChunkCoord { x: -1, y: 0 },
            StoreChunkCoord { x: 1, y: 0 },
            StoreChunkCoord { x: -1, y: 1 },
            StoreChunkCoord { x: 0, y: 1 },
        ] {
            chunks.insert(coord, data());
        }
        assert!(would_create_hole(&chunks, StoreChunkCoord { x: 1, y: 1 }));
    }
}
