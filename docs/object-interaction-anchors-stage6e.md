# Object Interaction Anchors (Stage 6E)

## Overview
Stage 6E enables objects like shelves and checkouts to generate functional `NpcAnchor` points from their `NpcInteractionPointsSpec`.

## Key Concepts
- **Interaction Points**: Object-authored anchor specifications that define where NPCs should stand and face when interacting with an object.
- **Semantic Kinds**: Anchors are tagged with specific kinds like `BrowseProducts` or `CheckoutCustomer`.
- **Target Resolution**: NPCs can target an object's interaction point by ID (`object_id` + `point_id`).
- **Preferred Animations**: Interaction points can specify which animation the NPC should play (e.g., `customer.browse_shelf`).

## Usage
Interaction points are defined in the object prototype.

### Shelf Prototype Example
```rust
ObjectCapabilitySpec::NpcInteractionPoints(NpcInteractionPointsSpec {
    points: vec![NpcInteractionPointSpec {
        id: "browse".to_string(),
        kind: NpcAnchorKind::BrowseProducts,
        local_offset: Vec2::new(0.0, 32.0),
        facing: Some(NpcDirection::S),
        allowed_roles: vec![NpcRole::Customer],
        reservation_policy: AnchorReservationPolicy::None,
        preferred_animation: Some(NpcAnimActionId("customer.browse_shelf".to_string())),
    }],
})
```

### Job Targeting
```rust
PushNpcJobRequested {
    npc: entity,
    queue: NpcJobQueueKind::Personal,
    job: NpcJob {
        kind: NpcJobKindId("base.move_to".to_string()),
        payload: NpcJobPayload::MoveTo { 
            target: NpcMoveTarget::InteractionPoint { 
                object_id: shelf_id, 
                point_id: AnchorLocalId("browse") 
            } 
        },
        requested_animation: None,
        source: NpcJobSource::Player,
    },
}
```
