use bevy::prelude::*;
use super::graph::NavigationGraph;

#[derive(Resource, Default)]
pub struct NavigationDebugSettings {
    pub show_graph: bool,
    pub show_anchors: bool,
}

pub fn draw_navigation_debug_system(
    settings: Res<NavigationDebugSettings>,
    graph: Res<NavigationGraph>,
    anchors: Res<crate::npc::anchor::NpcAnchorCache>,
    mut gizmos: Gizmos,
) {
    if settings.show_graph {
        for node in &graph.nodes {
            gizmos.circle_2d(node.pos, 4.0, Color::WHITE);
        }
        for edge in &graph.edges {
            let from = graph.nodes[edge.from as usize].pos;
            let to = graph.nodes[edge.to as usize].pos;
            gizmos.line_2d(from, to, Color::srgba(1.0, 1.0, 1.0, 0.2));
        }
    }

    if settings.show_anchors {
        for anchor in anchors.by_id.values() {
            let color = match anchor.kind {
                crate::npc::anchor::NpcAnchorKind::PortalInteriorSide => Color::srgb(0.0, 1.0, 0.0),
                crate::npc::anchor::NpcAnchorKind::PortalExteriorSide => Color::srgb(0.0, 0.5, 1.0),
                crate::npc::anchor::NpcAnchorKind::BrowseProducts => Color::srgb(1.0, 0.8, 0.0),
                crate::npc::anchor::NpcAnchorKind::CheckoutCustomer => Color::srgb(1.0, 0.0, 0.0),
                _ => Color::srgb(0.5, 0.5, 0.5),
            };
            gizmos.circle_2d(anchor.point.pos, 6.0, color);
            
            if let Some(_facing) = anchor.facing {
                // Draw facing arrow if available
                // Simplification for now
            }
        }
    }
}
