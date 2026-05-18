use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRouteSpace {
    World,
    StoreLocal,
}

#[derive(Component, Default)]
pub struct NpcRoute {
    pub space: NpcRouteSpace,
    pub waypoints: VecDeque<Vec2>,
}

pub enum RouteAxisOrder {
    XThenY,
    YThenX,
}

pub fn build_manhattan_route(
    start: Vec2,
    end: Vec2,
    order: RouteAxisOrder,
) -> VecDeque<Vec2> {
    let mut waypoints = VecDeque::new();
    
    if (start - end).length_squared() < 1e-6 {
        return waypoints;
    }

    match order {
        RouteAxisOrder::XThenY => {
            if (start.x - end.x).abs() > 1e-3 {
                waypoints.push_back(Vec2::new(end.x, start.y));
            }
            waypoints.push_back(end);
        }
        RouteAxisOrder::YThenX => {
            if (start.y - end.y).abs() > 1e-3 {
                waypoints.push_back(Vec2::new(start.x, end.y));
            }
            waypoints.push_back(end);
        }
    }

    waypoints
}

impl Default for NpcRouteSpace {
    fn default() -> Self {
        Self::World
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_route() {
        let start = Vec2::ZERO;
        let end = Vec2::new(10.0, 5.0);

        let route_x_y = build_manhattan_route(start, end, RouteAxisOrder::XThenY);
        assert_eq!(route_x_y.len(), 2);
        assert_eq!(route_x_y[0], Vec2::new(10.0, 0.0));
        assert_eq!(route_x_y[1], Vec2::new(10.0, 5.0));

        let route_y_x = build_manhattan_route(start, end, RouteAxisOrder::YThenX);
        assert_eq!(route_y_x.len(), 2);
        assert_eq!(route_y_x[0], Vec2::new(0.0, 5.0));
        assert_eq!(route_y_x[1], Vec2::new(10.0, 5.0));
    }

    #[test]
    fn test_empty_route() {
        let start = Vec2::new(1.0, 1.0);
        let end = Vec2::new(1.0, 1.0);
        let route = build_manhattan_route(start, end, RouteAxisOrder::XThenY);
        assert!(route.is_empty());
    }
}
