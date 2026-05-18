use crate::objects::prototypes::{WallOpeningAnchor, WallOpeningShapeSpec, WallOpeningSpec};

/// Resolved wall-local rect for a single opening.
/// `offset_*` are along the segment; `height_*` are vertical (0 = floor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallOpeningRect {
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}

/// Reserved runtime shape enum. `Polygon` variant exists for future extension but
/// is never constructed in Stage 5B.6 — Polygon specs are rejected at catalog validation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WallOpeningShape {
    Rect(WallOpeningRect),
    /// Reserved. Not constructed in 5B.6.
    #[allow(dead_code)]
    Polygon(WallOpeningPolygon),
}

/// Polygon opening in wall-local coordinates (x = offset, y = height).
/// Defined as a type so future backends can match on `WallOpeningShape::Polygon`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WallOpeningPolygon {
    pub vertices: Vec<bevy::math::Vec2>,
}

/// Error from `derive_opening_rect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallOpeningDeriveError {
    /// Polygon shape is not supported by the current Rect backend.
    /// No panic, no silent bounding-rect fallback.
    UnsupportedShape,
}

/// Derive a `WallOpeningRect` from an object's attachment and its `WallOpeningSpec`.
///
/// Returns `Err(UnsupportedShape)` if the spec uses `Polygon`.
pub fn derive_opening_rect(
    offset_along_segment: f32,
    height_on_wall: f32,
    spec: &WallOpeningSpec,
) -> Result<WallOpeningRect, WallOpeningDeriveError> {
    match &spec.shape {
        WallOpeningShapeSpec::Rect { width, height, anchor } => {
            let half_w = width * 0.5;
            let offset_min = offset_along_segment - half_w;
            let offset_max = offset_along_segment + half_w;
            let (height_min, height_max) = match anchor {
                WallOpeningAnchor::Center => {
                    let half_h = height * 0.5;
                    (height_on_wall - half_h, height_on_wall + half_h)
                }
                WallOpeningAnchor::BottomCenter => (height_on_wall, height_on_wall + height),
            };
            Ok(WallOpeningRect { offset_min, offset_max, height_min, height_max })
        }
        WallOpeningShapeSpec::Polygon { .. } => Err(WallOpeningDeriveError::UnsupportedShape),
    }
}

/// Error from `validate_opening`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WallOpeningValidationError {
    ZeroOrNegativeDimension,
    ExceedsSurfaceLength,
    ExceedsSurfaceHeight,
}

/// Validate a `WallOpeningRect` against the wall surface dimensions.
#[allow(dead_code)]
pub fn validate_opening(
    opening: &WallOpeningRect,
    surface_length: f32,
    surface_height: f32,
) -> Result<(), WallOpeningValidationError> {
    if opening.offset_min >= opening.offset_max || opening.height_min >= opening.height_max {
        return Err(WallOpeningValidationError::ZeroOrNegativeDimension);
    }
    if opening.offset_min < 0.0 || opening.offset_max > surface_length {
        return Err(WallOpeningValidationError::ExceedsSurfaceLength);
    }
    if opening.height_min < 0.0 || opening.height_max > surface_height {
        return Err(WallOpeningValidationError::ExceedsSurfaceHeight);
    }
    Ok(())
}

/// A solid rectangular piece of wall that should be rendered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallPieceRect {
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}

/// Split the full wall surface `[0, surface_length] x [0, surface_height]` into solid
/// `WallPieceRect`s by cutting out the given openings.
///
/// Overlapping openings are merged horizontally before splitting.
/// If `openings` is empty, returns one rect covering the full wall.
pub fn split_wall_around_openings(
    surface_length: f32,
    surface_height: f32,
    openings: &[WallOpeningRect],
) -> Vec<WallPieceRect> {
    if openings.is_empty() {
        return vec![WallPieceRect {
            offset_min: 0.0,
            offset_max: surface_length,
            height_min: 0.0,
            height_max: surface_height,
        }];
    }

    // Sort by offset_min, then merge overlapping horizontal intervals.
    let mut sorted: Vec<WallOpeningRect> = openings.to_vec();
    sorted.sort_by(|a, b| a.offset_min.partial_cmp(&b.offset_min).unwrap());

    // We process each opening independently, keeping vertical bands intact.
    // For each opening we emit up to 4 pieces: left strip, right strip (full-height),
    // bottom piece, top piece (between opening horizontal extents).
    // Left/right strips that would duplicate from adjacent openings are deduped by
    // sweep: only emit the left strip up to the current opening's left edge.

    let mut pieces: Vec<WallPieceRect> = Vec::new();
    let mut cursor = 0.0f32; // tracks the right edge of the last emitted full-height strip

    for opening in &sorted {
        let left = opening.offset_min.max(0.0);
        let right = opening.offset_max.min(surface_length);
        let bot = opening.height_min.max(0.0);
        let top = opening.height_max.min(surface_height);

        // Left full-height strip (from cursor to opening left edge)
        if cursor < left {
            pieces.push(WallPieceRect {
                offset_min: cursor,
                offset_max: left,
                height_min: 0.0,
                height_max: surface_height,
            });
        }
        cursor = cursor.max(right);

        // Bottom piece (below opening)
        if bot > 0.0 {
            pieces.push(WallPieceRect {
                offset_min: left,
                offset_max: right,
                height_min: 0.0,
                height_max: bot,
            });
        }

        // Top piece (above opening)
        if top < surface_height {
            pieces.push(WallPieceRect {
                offset_min: left,
                offset_max: right,
                height_min: top,
                height_max: surface_height,
            });
        }
        // The opening area itself has no piece.
    }

    // Right full-height strip (after last opening)
    if cursor < surface_length {
        pieces.push(WallPieceRect {
            offset_min: cursor,
            offset_max: surface_length,
            height_min: 0.0,
            height_max: surface_height,
        });
    }

    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::prototypes::{WallOpeningAnchor, WallOpeningShapeSpec, WallOpeningSpec};

    fn rect_spec(width: f32, height: f32, anchor: WallOpeningAnchor) -> WallOpeningSpec {
        WallOpeningSpec {
            shape: WallOpeningShapeSpec::Rect { width, height, anchor },
            glass_color: None,
            frame_color: None,
        }
    }

    fn polygon_spec() -> WallOpeningSpec {
        WallOpeningSpec {
            shape: WallOpeningShapeSpec::Polygon {
                vertices: vec![],
                anchor: WallOpeningAnchor::Center,
            },
            glass_color: None,
            frame_color: None,
        }
    }

    // --- derive_opening_rect ---

    #[test]
    fn derive_rect_center() {
        let spec = rect_spec(40.0, 20.0, WallOpeningAnchor::Center);
        let r = derive_opening_rect(100.0, 50.0, &spec).unwrap();
        assert_eq!(r.offset_min, 80.0);
        assert_eq!(r.offset_max, 120.0);
        assert_eq!(r.height_min, 40.0);
        assert_eq!(r.height_max, 60.0);
    }

    #[test]
    fn derive_rect_bottom_center() {
        let spec = rect_spec(64.0, 96.0, WallOpeningAnchor::BottomCenter);
        let r = derive_opening_rect(128.0, 0.0, &spec).unwrap();
        assert_eq!(r.offset_min, 96.0);
        assert_eq!(r.offset_max, 160.0);
        assert_eq!(r.height_min, 0.0);
        assert_eq!(r.height_max, 96.0);
    }

    #[test]
    fn derive_polygon_returns_unsupported() {
        let spec = polygon_spec();
        assert_eq!(
            derive_opening_rect(0.0, 0.0, &spec),
            Err(WallOpeningDeriveError::UnsupportedShape)
        );
    }

    // --- validate_opening ---

    #[test]
    fn validate_opening_inside() {
        let r = WallOpeningRect { offset_min: 10.0, offset_max: 50.0, height_min: 0.0, height_max: 80.0 };
        assert!(validate_opening(&r, 200.0, 120.0).is_ok());
    }

    #[test]
    fn validate_opening_exceeds_length() {
        let r = WallOpeningRect { offset_min: 180.0, offset_max: 220.0, height_min: 0.0, height_max: 80.0 };
        assert_eq!(validate_opening(&r, 200.0, 120.0), Err(WallOpeningValidationError::ExceedsSurfaceLength));
    }

    #[test]
    fn validate_opening_exceeds_height() {
        let r = WallOpeningRect { offset_min: 10.0, offset_max: 50.0, height_min: 0.0, height_max: 200.0 };
        assert_eq!(validate_opening(&r, 200.0, 120.0), Err(WallOpeningValidationError::ExceedsSurfaceHeight));
    }

    #[test]
    fn validate_opening_zero_width() {
        let r = WallOpeningRect { offset_min: 50.0, offset_max: 50.0, height_min: 0.0, height_max: 80.0 };
        assert_eq!(validate_opening(&r, 200.0, 120.0), Err(WallOpeningValidationError::ZeroOrNegativeDimension));
    }

    // --- split_wall_around_openings ---

    #[test]
    fn split_no_openings_returns_full_wall() {
        let pieces = split_wall_around_openings(200.0, 120.0, &[]);
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0], WallPieceRect { offset_min: 0.0, offset_max: 200.0, height_min: 0.0, height_max: 120.0 });
    }

    #[test]
    fn split_door_opening_floor_level() {
        // Door: floor-to-height, center segment. Expect: left strip, right strip, top piece.
        let opening = WallOpeningRect { offset_min: 60.0, offset_max: 124.0, height_min: 0.0, height_max: 96.0 };
        let pieces = split_wall_around_openings(200.0, 120.0, &[opening]);
        // left strip (full height 0..120)
        assert!(pieces.iter().any(|p| p.offset_min == 0.0 && p.offset_max == 60.0 && p.height_min == 0.0 && p.height_max == 120.0));
        // right strip (full height 0..120)
        assert!(pieces.iter().any(|p| p.offset_min == 124.0 && p.offset_max == 200.0 && p.height_min == 0.0 && p.height_max == 120.0));
        // top piece above door
        assert!(pieces.iter().any(|p| p.offset_min == 60.0 && p.offset_max == 124.0 && p.height_min == 96.0 && p.height_max == 120.0));
        // no bottom piece (door is floor-level)
        assert!(!pieces.iter().any(|p| p.offset_min == 60.0 && p.offset_max == 124.0 && p.height_max <= 0.0));
        assert_eq!(pieces.len(), 3);
    }

    #[test]
    fn split_window_opening_mid_height() {
        // Window: mid-height. Expect: left, right, bottom, top pieces.
        let opening = WallOpeningRect { offset_min: 60.0, offset_max: 132.0, height_min: 40.0, height_max: 88.0 };
        let pieces = split_wall_around_openings(200.0, 120.0, &[opening]);
        assert!(pieces.iter().any(|p| p.offset_min == 0.0 && p.offset_max == 60.0));   // left
        assert!(pieces.iter().any(|p| p.offset_min == 132.0 && p.offset_max == 200.0)); // right
        assert!(pieces.iter().any(|p| p.offset_min == 60.0 && p.offset_max == 132.0 && p.height_min == 0.0 && p.height_max == 40.0)); // bottom
        assert!(pieces.iter().any(|p| p.offset_min == 60.0 && p.offset_max == 132.0 && p.height_min == 88.0 && p.height_max == 120.0)); // top
        assert_eq!(pieces.len(), 4);
    }

    #[test]
    fn split_two_non_overlapping_openings() {
        let o1 = WallOpeningRect { offset_min: 20.0, offset_max: 60.0, height_min: 0.0, height_max: 80.0 };
        let o2 = WallOpeningRect { offset_min: 100.0, offset_max: 140.0, height_min: 0.0, height_max: 80.0 };
        let pieces = split_wall_around_openings(200.0, 120.0, &[o1, o2]);
        // left strip, inter-opening strip, right strip, top piece x2
        assert_eq!(pieces.len(), 5);
    }
}
