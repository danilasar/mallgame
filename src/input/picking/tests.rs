use super::*;
use crate::presentation::IsoProjection;
use crate::store::{StoreChunkCoord, WallSegmentKey, WallSurface};

#[test]
fn wall_surface_hit_projects_and_clamps_to_surface() {
    let projection = IsoProjection::default();
    let surface = WallSurface {
        key: WallSegmentKey {
            chunk: StoreChunkCoord { x: 0, y: 0 },
            side: crate::store::boundary::StoreBoundarySide::Top,
        },
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(4.0, 0.0),
        length: 4.0,
        height: 6.0,
        thickness: 2.0,
        normal: Vec2::Y,
    };

    let projected_start = world_to_iso(surface.start, projection);
    let projected_end = world_to_iso(surface.end, projection);
    let projected_mid = projected_start.lerp(projected_end, 0.5);
    let wall_direction = (projected_end - projected_start).normalize();
    let wall_normal = Vec2::new(-wall_direction.y, wall_direction.x);
    let projected_mid_on_face = projected_mid + wall_normal * surface.thickness;

    let hit = wall_surface_hit(
        projected_mid_on_face,
        projection,
        Entity::from_bits(1),
        &surface,
    )
    .expect("expected a wall hit");

    assert_eq!(hit.key, surface.key);
    assert!((hit.offset_along_segment - 2.0).abs() < 0.001);
    assert!(hit.height_on_wall >= 0.0 && hit.height_on_wall <= surface.height);
    assert_eq!(hit.normal, Vec2::Y);
    assert!((hit.world_pos - Vec2::new(2.0, 0.0)).length() < 0.001);

    let elevated = projected_mid_on_face + Vec2::new(0.0, 0.5);
    let elevated_hit = wall_surface_hit(elevated, projection, Entity::from_bits(2), &surface)
        .expect("expected elevated wall hit");
    assert!(elevated_hit.height_on_wall > 0.0);

    let wall_face = projected_mid_on_face + wall_normal * 0.5;
    assert!(wall_surface_hit(wall_face, projection, Entity::from_bits(4), &surface).is_some());

    let behind_wall = projected_mid_on_face - wall_normal * 2.5;
    assert!(wall_surface_hit(behind_wall, projection, Entity::from_bits(5), &surface).is_none());

    let outside = projected_mid_on_face + wall_direction * 5.0;
    assert!(wall_surface_hit(outside, projection, Entity::from_bits(3), &surface).is_none());
}
