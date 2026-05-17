use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct IsoProjection {
    pub x_scale: f32,
    pub y_scale: f32,
}

impl Default for IsoProjection {
    fn default() -> Self {
        Self {
            x_scale: 1.0,
            y_scale: 0.5,
        }
    }
}

pub fn world_to_iso(world: Vec2, projection: IsoProjection) -> Vec2 {
    Vec2::new(
        (world.x - world.y) * projection.x_scale,
        (world.x + world.y) * projection.y_scale,
    )
}

pub fn iso_to_world(projected: Vec2, projection: IsoProjection) -> Vec2 {
    let x_minus_y = projected.x / projection.x_scale;
    let x_plus_y = projected.y / projection.y_scale;

    Vec2::new((x_plus_y + x_minus_y) * 0.5, (x_plus_y - x_minus_y) * 0.5)
}
