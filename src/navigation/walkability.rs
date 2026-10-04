use bevy::prelude::*;
use crate::store::area::StoreArea;

/// Pure helper to check if a point is inside the walkable store area.
pub fn is_point_inside_store_area(pos: Vec2, store_area: &StoreArea) -> bool {
    // This depends on how StoreArea is implemented. 
    // Usually it's a set of chunks.
    store_area.contains_point(pos)
}

/// Pure helper to check if a point is blocked by a static blocker.
pub fn is_point_blocked_by_static(
    pos: Vec2,
    blockers: &[(Vec2, &crate::objects::components::Footprint)],
) -> bool {
    for (blocker_pos, footprint) in blockers {
        if is_point_in_footprint(pos, *blocker_pos, footprint) {
            return true;
        }
    }
    false
}

pub fn is_point_in_footprint(
    pos: Vec2,
    blocker_pos: Vec2,
    footprint: &crate::objects::components::Footprint,
) -> bool {
    let relative_pos = pos - blocker_pos;
    point_in_polygon(relative_pos, &footprint.local_polygon)
}

pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if ((polygon[i].y > point.y) != (polygon[j].y > point.y))
            && (point.x
                < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y)
                    / (polygon[j].y - polygon[i].y)
                    + polygon[i].x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::components::Footprint;

    #[test]
    fn test_point_in_polygon() {
        let poly = vec![
            Vec2::new(-10.0, -10.0),
            Vec2::new(10.0, -10.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(-10.0, 10.0),
        ];

        assert!(point_in_polygon(Vec2::ZERO, &poly));
        assert!(point_in_polygon(Vec2::new(5.0, 5.0), &poly));
        assert!(!point_in_polygon(Vec2::new(15.0, 0.0), &poly));
    }

    #[test]
    fn test_point_blocked_by_static() {
        let footprint = Footprint::rectangle(Vec2::new(10.0, 10.0));
        let blockers = vec![(Vec2::new(100.0, 100.0), &footprint)];

        assert!(is_point_blocked_by_static(Vec2::new(100.0, 100.0), &blockers));
        assert!(is_point_blocked_by_static(Vec2::new(105.0, 105.0), &blockers));
        assert!(!is_point_blocked_by_static(Vec2::ZERO, &blockers));
    }

    #[test]
    fn test_is_point_inside_store_area() {
        let anchor = Vec2::ZERO;
        let store = crate::store::area::StoreArea::new(anchor);
        
        // Initial store has chunks from -5..0, -4..0
        // chunk_world_size is 32 * 4 = 128
        // So rects from (-640, -512) to (0, 0)
        
        assert!(is_point_inside_store_area(Vec2::new(-10.0, -10.0), &store));
        assert!(is_point_inside_store_area(Vec2::new(-630.0, -500.0), &store));
        assert!(!is_point_inside_store_area(Vec2::new(10.0, 10.0), &store));
    }
}
