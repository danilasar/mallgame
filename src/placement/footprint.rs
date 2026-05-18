use bevy::prelude::*;

use crate::objects::components::Footprint;

#[derive(Debug, Clone, Copy)]
pub struct FootprintBounds {
    pub min: Vec2,
    pub max: Vec2,
}

pub fn world_polygon(footprint: &Footprint, world_pos: Vec2) -> Vec<Vec2> {
    footprint
        .local_polygon
        .iter()
        .map(|point| *point + world_pos)
        .collect()
}

pub fn polygon_bounds(points: &[Vec2]) -> Option<FootprintBounds> {
    let first = *points.first()?;
    let mut min = first;
    let mut max = first;

    for point in points.iter().copied().skip(1) {
        min = min.min(point);
        max = max.max(point);
    }

    Some(FootprintBounds { min, max })
}

pub fn footprints_intersect(
    a_footprint: &Footprint,
    a_world_pos: Vec2,
    b_footprint: &Footprint,
    b_world_pos: Vec2,
) -> bool {
    let a_world = world_polygon(a_footprint, a_world_pos);
    let b_world = world_polygon(b_footprint, b_world_pos);
    let Some(a) = polygon_bounds(&a_world) else {
        return false;
    };
    let Some(b) = polygon_bounds(&b_world) else {
        return false;
    };

    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

pub fn polygon_intersects_access_zone(
    polygon: &[Vec2],
    access_zone: &crate::objects::components::InteriorAccessZone,
) -> bool {
    let Some(a) = polygon_bounds(polygon) else {
        return false;
    };
    let Some(b) = polygon_bounds(&access_zone.polygon) else {
        return false;
    };

    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}
