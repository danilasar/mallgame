use bevy::prelude::*;

/// Continuous isometric projection from gameplay world space into 2D camera space.
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

pub fn cursor_to_projected(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

pub fn cursor_to_world(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    projection: IsoProjection,
) -> Option<Vec2> {
    cursor_to_projected(window, camera, camera_transform)
        .map(|projected| iso_to_world(projected, projection))
}
