use bevy::prelude::*;
use super::types::{RoutePlanRequest, RoutePlanResult, RoutePlanError, NavigationRoute};
use super::graph::{NavigationGraph, NavigationNodeId};
use std::collections::{VecDeque, HashMap, BinaryHeap};
use std::cmp::Ordering;

pub trait RoutePlanner {
    fn plan_route(&self, request: RoutePlanRequest) -> RoutePlanResult;
}

#[derive(Resource)]
pub struct NavigationPlanner;

impl RoutePlanner for NavigationGraph {
    fn plan_route(&self, request: RoutePlanRequest) -> RoutePlanResult {
        if request.start.space != self.space || request.target.space != self.space {
            return Err(RoutePlanError::UnsupportedSpace);
        }

        // 1. Find nearest nodes to start and target
        let start_node_id = self.find_nearest_node(request.start.pos)?;
        let target_node_id = self.find_nearest_node(request.target.pos)?;

        // 2. A* Search
        let path = self.a_star(start_node_id, target_node_id)?;

        // 3. Convert path to waypoints
        let mut waypoints = VecDeque::new();
        for node_id in path {
            waypoints.push_back(self.nodes[node_id as usize].pos);
        }
        
        // Ensure final target pos is reached exactly
        waypoints.push_back(request.target.pos);

        Ok(NavigationRoute {
            space: self.space,
            waypoints,
        })
    }
}

impl NavigationGraph {
    pub fn find_nearest_node(&self, pos: Vec2) -> Result<NavigationNodeId, RoutePlanError> {
        let mut best: Option<(f32, NavigationNodeId)> = None;
        for node in &self.nodes {
            let d = node.pos.distance_squared(pos);
            if best.as_ref().is_none_or(|(dist, _)| d < *dist) {
                best = Some((d, node.id));
            }
        }

        match best {
            Some((dist, id)) if dist < 64.0 * 64.0 => Ok(id), // 2 cells radius
            _ => Err(RoutePlanError::NoRoute),
        }
    }

    fn a_star(&self, start: NavigationNodeId, target: NavigationNodeId) -> Result<Vec<NavigationNodeId>, RoutePlanError> {
        let mut open_set = BinaryHeap::new();
        open_set.push(NodeScore {
            id: start,
            score: 0.0,
        });

        let mut came_from: HashMap<NavigationNodeId, NavigationNodeId> = HashMap::new();
        let mut g_score: HashMap<NavigationNodeId, f32> = HashMap::new();
        g_score.insert(start, 0.0);

        let mut f_score: HashMap<NavigationNodeId, f32> = HashMap::new();
        f_score.insert(start, self.nodes[start as usize].pos.distance(self.nodes[target as usize].pos));

        // Group edges by 'from' for efficiency
        let mut adjacency: HashMap<NavigationNodeId, Vec<(NavigationNodeId, f32)>> = HashMap::new();
        for edge in &self.edges {
            adjacency.entry(edge.from).or_default().push((edge.to, edge.cost));
        }

        while let Some(NodeScore { id: current, .. }) = open_set.pop() {
            if current == target {
                return Ok(reconstruct_path(came_from, current));
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for (neighbor, cost) in neighbors {
                    let tentative_g_score = g_score[&current] + cost;
                    if tentative_g_score < *g_score.get(neighbor).unwrap_or(&f32::INFINITY) {
                        came_from.insert(*neighbor, current);
                        g_score.insert(*neighbor, tentative_g_score);
                        let f = tentative_g_score + self.nodes[*neighbor as usize].pos.distance(self.nodes[target as usize].pos);
                        f_score.insert(*neighbor, f);
                        open_set.push(NodeScore {
                            id: *neighbor,
                            score: f,
                        });
                    }
                }
            }
        }

        Err(RoutePlanError::NoRoute)
    }
}

fn reconstruct_path(came_from: HashMap<NavigationNodeId, NavigationNodeId>, mut current: NavigationNodeId) -> Vec<NavigationNodeId> {
    let mut total_path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(current);
    }
    total_path.reverse();
    total_path
}

#[derive(Copy, Clone, PartialEq)]
struct NodeScore {
    id: NavigationNodeId,
    score: f32,
}

impl Eq for NodeScore {}

impl Ord for NodeScore {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.partial_cmp(&self.score).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for NodeScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::*;
    use super::super::graph::*;

    #[test]
    fn test_a_star_simple() {
        let mut graph = NavigationGraph::new(NavigationSpace::World);
        
        // Two nodes
        graph.nodes.push(NavigationNode { id: 0, pos: Vec2::ZERO });
        graph.nodes.push(NavigationNode { id: 1, pos: Vec2::new(100.0, 0.0) });
        
        // One edge
        graph.edges.push(NavigationEdge { from: 0, to: 1, cost: 100.0 });
        graph.edges.push(NavigationEdge { from: 1, to: 0, cost: 100.0 });

        let request = RoutePlanRequest {
            start: NavigationPoint { space: NavigationSpace::World, pos: Vec2::new(5.0, 0.0) },
            target: NavigationPoint { space: NavigationSpace::World, pos: Vec2::new(95.0, 0.0) },
            agent: AgentNavigationProfile { radius: 10.0, clearance: 2.0, can_use_staff_only: false },
        };

        let result = graph.plan_route(request);
        assert!(result.is_ok());
        let route = result.unwrap();
        assert_eq!(route.waypoints.len(), 3); // node 0, node 1, and final target pos
    }
}
