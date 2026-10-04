use bevy::prelude::*;
use super::types::NavigationSpace;
use crate::store::area::StoreArea;
use crate::objects::components::{WorldPos, Footprint, BlocksPlacement};
use std::collections::HashMap;

pub type NavigationNodeId = u32;

pub struct NavigationNode {
    pub id: NavigationNodeId,
    pub pos: Vec2,
}

pub struct NavigationEdge {
    pub from: NavigationNodeId,
    pub to: NavigationNodeId,
    pub cost: f32,
}

#[derive(Resource)]
pub struct NavigationGraph {
    pub space: NavigationSpace,
    pub nodes: Vec<NavigationNode>,
    pub edges: Vec<NavigationEdge>,
    pub node_map: HashMap<IVec2, NavigationNodeId>, // grid coord -> node id
    pub version: u64,
}

impl NavigationGraph {
    pub fn new(space: NavigationSpace) -> Self {
        Self {
            space,
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
            version: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationRegionKind {
    StoreInterior,
    ExteriorWorld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirtyReason {
    StoreAreaChanged,
    StaticBlockerBuilt,
    StaticBlockerMoved,
    StaticBlockerDeleted,
    WorldReset,
    LoadCompleted,
    DebugSettingsChanged,
}

#[derive(Resource, Default)]
pub struct NavigationDirtyState {
    pub reasons: Vec<NavigationDirtyReason>,
}

impl NavigationDirtyState {
    pub fn is_dirty(&self) -> bool {
        !self.reasons.is_empty()
    }

    pub fn clear(&mut self) {
        self.reasons.clear();
    }

    pub fn mark_dirty(&mut self, reason: NavigationDirtyReason) {
        self.reasons.push(reason);
    }
}

pub struct PortalEdgeData {
    pub anchor_a_pos: Vec2,
    pub anchor_b_pos: Vec2,
    pub bidirectional: bool,
}

pub fn rebuild_navigation_graph(
    store_area: &StoreArea,
    blockers: &[(Vec2, &Footprint)],
    portals: &[PortalEdgeData],
) -> NavigationGraph {
    let mut graph = NavigationGraph::new(NavigationSpace::World);
    let cell_size = 32.0;

    // 1. Identify walkable grid cells
    for (&coord, _data) in store_area.owned_chunks.iter() {
        let chunk_rect = store_area.chunk_rect(coord);
        for gy in 0..store_area.chunk_size_cells.y {
            for gx in 0..store_area.chunk_size_cells.x {
                let cell_pos =
                    chunk_rect.min + Vec2::new(gx as f32 + 0.5, gy as f32 + 0.5) * cell_size;

                if crate::navigation::walkability::is_point_blocked_by_static(cell_pos, blockers) {
                    continue;
                }

                let grid_coord = IVec2::new(
                    ((cell_pos.x - store_area.anchor.x) / cell_size).floor() as i32,
                    ((cell_pos.y - store_area.anchor.y) / cell_size).floor() as i32,
                );
                let id = graph.nodes.len() as NavigationNodeId;
                graph.nodes.push(NavigationNode { id, pos: cell_pos });
                graph.node_map.insert(grid_coord, id);
            }
        }
    }

    // 2. Create edges between adjacent walkable cells
    let nodes_to_process: Vec<(IVec2, NavigationNodeId)> =
        graph.node_map.iter().map(|(&k, &v)| (k, v)).collect();
    for (grid_coord, from_id) in nodes_to_process {
        for neighbor_offset in [
            IVec2::new(1, 0),
            IVec2::new(-1, 0),
            IVec2::new(0, 1),
            IVec2::new(0, -1),
        ] {
            let neighbor_coord = grid_coord + neighbor_offset;
            if let Some(&to_id) = graph.node_map.get(&neighbor_coord) {
                graph.edges.push(NavigationEdge {
                    from: from_id,
                    to: to_id,
                    cost: cell_size,
                });
            }
        }
    }

    // 3. Add Portal Edges
    for portal in portals {
        if let Some(node_a) = find_nearest_node_id_local(&graph, portal.anchor_a_pos)
            && let Some(node_b) = find_nearest_node_id_local(&graph, portal.anchor_b_pos)
        {
            let dist = portal.anchor_a_pos.distance(portal.anchor_b_pos);
            graph.edges.push(NavigationEdge {
                from: node_a,
                to: node_b,
                cost: dist,
            });
            if portal.bidirectional {
                graph.edges.push(NavigationEdge {
                    from: node_b,
                    to: node_a,
                    cost: dist,
                });
            }
        }
    }

    graph.version = 1;
    graph
}

fn find_nearest_node_id_local(graph: &NavigationGraph, pos: Vec2) -> Option<NavigationNodeId> {
    let mut best: Option<(f32, NavigationNodeId)> = None;
    for node in &graph.nodes {
        let d = node.pos.distance_squared(pos);
        if best.as_ref().is_none_or(|(dist, _)| d < *dist) {
            best = Some((d, node.id));
        }
    }
    match best {
        Some((dist, id)) if dist < 64.0 * 64.0 => Some(id),
        _ => None,
    }
}

// Local helper for graph building to avoid circular dependency or complex Query passing
fn is_point_blocked_by_static_local(
    pos: Vec2,
    blockers: &Query<(&WorldPos, &Footprint), With<BlocksPlacement>>,
) -> bool {
    for (blocker_pos, footprint) in blockers.iter() {
        let relative_pos = pos - blocker_pos.0;
        if point_in_polygon_local(relative_pos, &footprint.local_polygon) {
            return true;
        }
    }
    false
}

fn point_in_polygon_local(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if ((polygon[i].y > point.y) != (polygon[j].y > point.y)) &&
            (point.x < (polygon[j].x - polygon[i].x) * (point.y - polygon[i].y) / (polygon[j].y - polygon[i].y) + polygon[i].x) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
