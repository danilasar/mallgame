use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcDirection {
    E,
    N,
    NW,
    S,
    SE,
    W,
    SW,
    NE,
}

pub struct NpcDirectionMapping {
    pub positive_x: NpcDirection,
    pub negative_x: NpcDirection,
    pub positive_y: NpcDirection,
    pub negative_y: NpcDirection,
}

impl Default for NpcDirectionMapping {
    fn default() -> Self {
        // Default mapping assuming standard world coordinates:
        // +X is E/SE-ish, +Y is N/NE-ish in world? 
        // Let's use a simple axis-aligned mapping for Stage 6A.
        Self {
            positive_x: NpcDirection::E,
            negative_x: NpcDirection::W,
            positive_y: NpcDirection::N,
            negative_y: NpcDirection::S,
        }
    }
}

pub fn npc_direction_from_delta(
    delta: Vec2,
    mapping: &NpcDirectionMapping,
) -> Option<NpcDirection> {
    if delta.length_squared() < 1e-6 {
        return None;
    }

    // Manhattan-like or major axis detection
    if delta.x.abs() > delta.y.abs() {
        if delta.x > 0.0 {
            Some(mapping.positive_x)
        } else {
            Some(mapping.negative_x)
        }
    } else {
        if delta.y > 0.0 {
            Some(mapping.positive_y)
        } else {
            Some(mapping.negative_y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_from_delta() {
        let mapping = NpcDirectionMapping::default();

        assert_eq!(npc_direction_from_delta(Vec2::new(1.0, 0.0), &mapping), Some(NpcDirection::E));
        assert_eq!(npc_direction_from_delta(Vec2::new(-1.0, 0.0), &mapping), Some(NpcDirection::W));
        assert_eq!(npc_direction_from_delta(Vec2::new(0.0, 1.0), &mapping), Some(NpcDirection::N));
        assert_eq!(npc_direction_from_delta(Vec2::new(0.0, -1.0), &mapping), Some(NpcDirection::S));

        // Diagonal should pick major axis
        assert_eq!(npc_direction_from_delta(Vec2::new(1.0, 0.1), &mapping), Some(NpcDirection::E));
        assert_eq!(npc_direction_from_delta(Vec2::new(0.1, 1.0), &mapping), Some(NpcDirection::N));

        // Zero delta
        assert_eq!(npc_direction_from_delta(Vec2::ZERO, &mapping), None);
    }
}
