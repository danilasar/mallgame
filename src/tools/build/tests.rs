use super::*;
use crate::input::WallSurfaceHit;
use crate::store::{StoreBoundarySide, StoreChunkCoord, WallSegmentKey};

#[test]
fn wall_attachment_from_hit_clamps_to_surface_bounds() {
    let hit = WallSurfaceHit {
        entity: Entity::from_bits(1),
        key: WallSegmentKey {
            chunk: StoreChunkCoord { x: 0, y: 0 },
            side: StoreBoundarySide::Top,
        },
        world_pos: Vec2::new(3.0, 7.0),
        offset_along_segment: 0.2,
        height_on_wall: 20.0,
        normal: Vec2::Y,
    };

    let attachment = wall_attachment_from_hit(hit, 1.0, 4.0, 6.0);

    assert_eq!(attachment.segment_key, hit.key);
    assert!((attachment.offset_along_segment - 1.0).abs() < 0.001);
    assert!((attachment.height_on_wall - 6.0).abs() < 0.001);
}
