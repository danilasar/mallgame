pub mod types;
pub mod graph;
pub mod planner;
pub mod walkability;
pub mod portal;
pub mod debug;

use bevy::prelude::*;
use crate::store::events::DomainEvent;
use self::graph::{NavigationGraph, NavigationDirtyState, NavigationDirtyReason, rebuild_navigation_graph};
use self::types::NavigationSpace;
use self::debug::{NavigationDebugSettings, draw_navigation_debug_system};

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavigationGraph::new(NavigationSpace::World))
            .init_resource::<NavigationDirtyState>()
            .init_resource::<NavigationDebugSettings>()
            .add_systems(Update, (
                mark_navigation_dirty_system,
                rebuild_navigation_graph_system,
                draw_navigation_debug_system,
            ).chain());
    }
}

pub fn mark_navigation_dirty_system(
    mut events: MessageReader<DomainEvent>,
    mut dirty_state: ResMut<NavigationDirtyState>,
) {
    for event in events.read() {
        match event {
            DomainEvent::ObjectBuilt { .. } |
            DomainEvent::ObjectMoved { .. } |
            DomainEvent::ObjectDeleted { .. } => {
                dirty_state.mark_dirty(NavigationDirtyReason::StaticBlockerBuilt);
            }
            DomainEvent::ChunkPurchased { .. } |
            DomainEvent::StoreAreaChanged { .. } => {
                dirty_state.mark_dirty(NavigationDirtyReason::StoreAreaChanged);
            }
            _ => {}
        }
    }
}

pub fn rebuild_navigation_graph_system(
    mut graph: ResMut<NavigationGraph>,
    mut dirty_state: ResMut<NavigationDirtyState>,
    store_area: Res<crate::store::area::StoreArea>,
    blockers_query: Query<
        (&crate::objects::components::WorldPos, &crate::objects::components::Footprint),
        With<crate::objects::components::BlocksPlacement>,
    >,
    portals_query: Query<&self::portal::NavigationPortal>,
    anchors_cache: Res<crate::npc::anchor::NpcAnchorCache>,
) {
    if !dirty_state.is_dirty() && graph.version > 0 {
        return;
    }

    info!(
        "Rebuilding navigation graph... reasons: {:?}",
        dirty_state.reasons
    );

    let blockers: Vec<(Vec2, &crate::objects::components::Footprint)> = blockers_query
        .iter()
        .map(|(pos, footprint)| (pos.0, footprint))
        .collect();

    let mut portals = Vec::new();
    for portal in portals_query.iter() {
        if let Some(anchor_a) = anchors_cache.by_id.get(&portal.a)
            && let Some(anchor_b) = anchors_cache.by_id.get(&portal.b)
        {
            portals.push(self::graph::PortalEdgeData {
                anchor_a_pos: anchor_a.point.pos,
                anchor_b_pos: anchor_b.point.pos,
                bidirectional: portal.traversal_policy.bidirectional,
            });
        }
    }

    *graph = rebuild_navigation_graph(&store_area, &blockers, &portals);
    dirty_state.clear();
}
