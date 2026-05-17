use bevy::prelude::*;

use crate::components::{CollisionFootprint, Velocity, WorldPos};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlacementPolygon {
    pub points: Vec<Vec2>,
}

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub struct PlacementAreas {
    pub allowed: Vec<PlacementPolygon>,
    pub blocked: Vec<PlacementPolygon>,
}

pub fn movement_system(time: Res<Time>, mut query: Query<(&mut WorldPos, &Velocity)>) {
    for (mut world_pos, velocity) in &mut query {
        world_pos.0 += velocity.0 * time.delta_secs();
    }
}

pub fn footprints_overlap(
    a_pos: Vec2,
    a_footprint: CollisionFootprint,
    b_pos: Vec2,
    b_footprint: CollisionFootprint,
) -> bool {
    let delta = (a_pos - b_pos).abs();
    delta.x < a_footprint.half_extents.x + b_footprint.half_extents.x
        && delta.y < a_footprint.half_extents.y + b_footprint.half_extents.y
}
