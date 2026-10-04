# Doorway Portal Integration (Stage 6D)

## Overview
Stage 6D connects doorways to the navigation system using the `NpcAnchor` model and `NavigationPortal` edges.

## Key Concepts
- **NavigationPortal**: Represents an explicit connection between two anchors (interior and exterior sides of a door).
- **Portal Spec**: Prototypes can opt-in to portal capability via `NavigationPortalSpec`.
- **Graph Integration**: The navigation graph builder adds synthetic edges between portal anchors, allowing NPCs to "traverse" doors.
- **Role Policy**: Portals can be restricted to specific roles (e.g., `StaffOnlyDoor`).

## Usage
Doorways with the `NavigationPortal` capability automatically generate portals.

### Prototype Definition
```rust
ObjectCapabilitySpec::NavigationPortal(NavigationPortalSpec {
    kind: NavigationPortalKind::Doorway,
    interior_local_offset: Vec2::new(0.0, 32.0),
    exterior_local_offset: Vec2::new(0.0, -32.0),
    allowed_roles: vec![NpcRole::Customer, NpcRole::Staff],
    bidirectional: true,
})
```

## Debugging
Portal anchors are visible when `show_anchors` is enabled. Portal edges appear in the graph debug view.
