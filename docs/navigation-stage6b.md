# Navigation Substrate (Stage 6B)

## Overview
Stage 6B introduces the navigation substrate, which provides walkability information and route planning for NPCs.

## Key Concepts
- **Walkability**: Determined by the `StoreArea` (owned chunks) and static floor blockers (objects with `BlocksPlacement`).
- **NavigationGraph**: A sampled lattice graph that covers the walkable area. Nodes are sampled every 32 units.
- **RoutePlanner**: An A* implementation that finds the shortest path between two points in continuous world coordinates.
- **NavigationDirtyState**: Tracks changes to the environment (building, moving, deleting objects) and triggers deferred graph rebuilds.

## Usage
The `NavigationPlugin` manages the graph and updates it automatically. NPCs use the `NavigationGraph` resource to plan routes during job execution.

### Route Planning
To plan a route:
```rust
let request = RoutePlanRequest {
    start: NavigationPoint { space: NavigationSpace::World, pos: current_pos },
    target: NavigationPoint { space: NavigationSpace::World, pos: target_pos },
    agent: AgentNavigationProfile { radius: 16.0, clearance: 4.0, can_use_staff_only: false },
};

if let Ok(route) = graph.plan_route(request) {
    // Follow route.waypoints
}
```

## Debugging
Enable debug overlays in `NavigationDebugSettings` to visualize the graph nodes and edges.
