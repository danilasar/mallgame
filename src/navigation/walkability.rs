use bevy::prelude::*;
use crate::store::area::StoreArea;
use crate::objects::components::WorldPos;

/// Pure helper to check if a point is inside the walkable store area.
pub fn is_point_inside_store_area(pos: Vec2, store_area: &StoreArea) -> bool {
    // This depends on how StoreArea is implemented. 
    // Usually it's a set of chunks.
    store_area.contains_point(pos)
}

/// Pure helper to check if a point is blocked by a static blocker.
/// For Stage 6B, we assume static blockers have a footprint or radius.
pub fn is_point_blocked_by_static(
    pos: Vec2,
    blockers: &Query<(&WorldPos, &crate::objects::components::Footprint), With<crate::objects::components::BlocksPlacement>>,
) -> bool {
    for (blocker_pos, footprint) in blockers.iter() {
        // Simple bounding box or polygon check
        if is_point_in_footprint(pos, blocker_pos.0, footprint) {
            return true;
        }
    }
    false
}

fn is_point_in_footprint(pos: Vec2, blocker_pos: Vec2, footprint: &crate::objects::components::Footprint) -> bool {
    let relative_pos = pos - blocker_pos;
    
    // For Stage 6B MVP, we can use a simple point-in-polygon check.
    // The Footprint component has local_polygon.
    point_in_polygon(relative_pos, &footprint.local_polygon)
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if ((polygon[i].y > point.y) != (polygon[j].y > point.y)) &&
            (point.x < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y) / (polygon[j].y - polygon[i].y) + polygon[i].x) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
