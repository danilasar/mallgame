# NPC Agent Foundation (Stage 6A)

## Overview
Stage 6A implements the foundational systems for NPC agents (customers, staff, service NPC). It provides a robust data-driven model for movement, animation, and task execution.

## Key Features
- **NpcArchetypeSpec:** Centralized data model for NPC characteristics, visual specs, and role-based task profiles.
- **Direction Model:** 8-direction support (E, N, NW, S, SE, W, SW, NE) with mirror support for W/SW/NE assets.
- **Task Queues:** 
  - `PersonalTaskQueue`: For all NPCs, handles self-driven or player-assigned personal tasks.
  - `AssignedTaskQueue`: For staff/service roles, handles job-related tasks.
- **Locomotion:** Smooth movement along Manhattan routes in World coordinates.
- **Picking Integration:** NPCs are pickable via `InteractionRole::Npc`, integrated into the standard `PointerTargets` system.
- **Animation Resolution:** Translates action IDs and directions into specific animation clips based on archetype rules.

## Technical Details
- **Modules:** `src/npc/` contains all NPC-related logic.
- **Plugin:** `NpcPlugin` manages registration of resources, events, and systems.
- **Components:**
  - `Npc`: Marker for NPC root entities.
  - `NpcIdentity`: Stores stable ID, archetype ID, and role.
  - `WorldPos`: Simulation position (feet anchor).
  - `Facing`: Current movement direction.
  - `NpcLocomotion`: Movement state and speed.
  - `NpcAnimationPlayer`: Manages visual child animation.
- **Picking:** Uses `NpcPickBounds` for precise pointer targeting.

## Usage
To spawn an NPC, emit a `SpawnNpcRequested` event:
```rust
events.send(SpawnNpcRequested {
    archetype_id: NpcArchetypeId("customer.standard".to_string()),
    world_pos: Vec2::new(100.0, 100.0),
    role_override: None,
});
```

To assign a task:
```rust
events.send(PushNpcTaskRequested {
    npc: entity,
    queue: NpcTaskQueueKind::Personal,
    task: NpcTask {
        kind: NpcTaskKindId("base.move_to".to_string()),
        payload: NpcTaskPayload::MoveTo { target: NpcMoveTarget::Point(target_pos) },
        requested_animation: None,
        source: NpcTaskSource::Player,
    },
});
```
