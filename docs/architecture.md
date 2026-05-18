# Architecture

This project is a Rust + Bevy 2D isometric prototype for a freeform store builder/editor-like game.

## Runtime Layers

- `InputActions` snapshot physical keyboard/mouse input into gameplay actions.
- `ToolInputGate` converts input, pointer ownership, and modal/UI blocking into safe tool signals.
- `ToolRuntime` owns active tool mode and unfinished tool sessions.
- `UiRuntime` owns right dock, ribbon, modal stack, camera controls, and world widgets.
- `Domain systems` mutate gameplay truth via validated `DomainCommands`.
- `Presentation` derives overlays, highlights, transforms, and depth.

## Main Flow

1. `update_input_action_state`
2. `update_pointer_context`
3. `update_pointer_over_ui`
4. `update_hovered_object`
5. `update_tool_input_gate`
6. modal/UI request systems
7. `camera_drag_system`
8. tool systems
9. Request-to-Command conversion
10. `apply_domain_commands` (Atomic mutation)
11. `ApplyDeferred` (Bevy entity sync)
12. Domain event consumers (`PostDomainApply`)
13. presentation sync and overlays

## Runtime Performance Notes

- `StoreOverlay` keeps a cache of chunk overlay entities and only refreshes the previous/current hovered expansion chunk instead of rebuilding all overlays every frame.
- `Highlight` updates are delta-driven: only previously/currently relevant entities are touched, not every `Interactive` entity in the world.
- `apply_domain_commands` is still the largest maintainability hotspot in the domain layer, but it is isolated behind the command boundary.

## Module Map

- `src/main.rs`: app setup and plugin order.
- `src/input`: raw input, pointer conversion, drag state, and picking.
- `src/tools`: tool modes, pointer gate, sessions, previews, and tool-specific systems.
- `src/ui`: UI layers, modals, right dock, ribbon, camera controls, world widgets, and object inspector.
- `src/store`: world bounds, store chunks, expansion validation, store overlays, and domain commands/events.
- `src/placement`: footprint geometry and placement validation.
- `src/presentation`: projection, transform sync, highlights, and footprint overlays.
- `src/objects`: shared components, prototypes, and rotation data.
- `src/save`: authority-based save/load model.

## Key Runtime Types

- `WorldPos` is simulation position.
- `ProjectedPos` is isometric projection result.
- `FootAnchor` marks sorting/picking anchor.
- `Footprint` is local collision/placement geometry.
- `StoreObject` marks entities that live inside the store and can be moved or deleted.
- `ObjectStableId` is a persistent ID for objects across save/load.
- `Rotatable` carries rotation variants for sprite, footprint, foot anchor, and visual offset.
- `StoreArea` owns bought `4x4` chunks.
- `WorldBounds` owns the outer world rectangle.
- `ToolContext` stores hovered entity, pointer coordinates, and active session data.
- `ToolSessionState` stores current build/move/expansion sessions.
- `ModalStack` stores modal lifecycle and blocking state.
- `PointerTargets` stores hovered world object/widget/wall/exterior/debug target separation.
- `PrimaryPointerCycle` stores primary click ownership across frame boundaries.
- `SelectionState` owns currently focused objects.

## Requests And Events

These are boundaries, not direct gameplay mutations:

- `ActivateToolRequested`
- `ReturnToPreviousToolRequested`
- `SelectBuildObjectRequested`
- `StartMoveObjectRequested`
- `MoveObjectCommitted`
- `BuildObjectRequested`
- `DeleteObjectRequested`
- `RotateObjectRequested`
- `ModalRequest`
- `PurchaseStoreChunkRequested`
- `CameraControlRequested`

## Domain Commands & Events

Gameplay mutation authority:

- `DomainCommand`: Encapsulates a validated domain operation (Build, Move, Rotate, Delete, Purchase).
- `DomainEvent`: Represents a fact that occurred after a successful mutation (ObjectBuilt, ObjectMoved, etc.).
- `DomainCommandQueue`: Deterministic queue for processing mutations in frame sequence.

## Authority Save Model

- `SaveGame` persists gameplay truth only: store chunks, real objects, stable IDs, prototype IDs, and positions.
- Transient runtime state (sessions, previews, widgets, selection, modal stack) is NOT saved.
- Load pipeline is transactional: validation occurs before clearing the current world.
- Runtime state is fully reset during load to ensure consistency.

## Current Implementation Notes

- Tool sessions are preview-based for build and move.
- Selection and a real `ObjectInspector` are implemented.
- The active build selection surface is the Ribbon; build-panel terminology is obsolete in the current codebase.
- Stage 5A catalog is already componentized: `ObjectPrototype` carries `display`, `catalog`, `placement`, `visuals`, `rotation`, `capabilities`, and `initial_state`.
- `ObjectCatalogSpec` is the Ribbon-facing metadata layer: `category`, `ribbon_tab`, `ribbon_group`, `sort_order`, and `availability`.
- Stage 5A capabilities currently map to typed ECS components: `ProductContainer`, `CheckoutPoint`, `Decor`, and `NpcInteractionPoints`.
- `NpcInteractionPoints` is a placeholder affordance layer for future NPCs, not NPC logic.
- Current sample prototypes are `fixture.shelf.basic`, `service.checkout.basic`, and `decor.plant.tree`, with save/load compatibility aliases `chair`, `table`, and `tree`.
- Catalog validation checks for non-empty display names and capability/interaction-point invariants:
  - `ProductContainer` requires at least one `BrowseProducts` NPC interaction point.
  - `CheckoutPoint` requires at least one `Checkout` NPC interaction point.
- Stage 5B.1 introduces a derived boundary-wall layer and an exterior-layer foundation inside `WorldBounds`; exterior is not background-only decor, but it still is not `StoreObject`.
- Store walls are derived from the outer top-right owned chunk of `StoreArea + expansion policy`, extend along the top row and right column while the run remains contiguous, are not save authority, and are synced through a `WallVisualCache`.
- Wall entities carry `StoreWallSegment` and `WallSurface` metadata; `WallSurface` stores `start/end`, length, thickness, height, and normal for future wall tools.
- `PointerTargets` now has dedicated `wall_surface` and `exterior` slots for safe picking separation.
- Stage 5B.2 adds `PlacementKind::WallMounted` as a preview-only branch in `BuildTool`: wall-mounted prototypes use `PointerTargets.wall_surface` and `WallAttachmentPoint`, but do not yet spawn real wall-mounted objects or domain commands.
- The Ribbon now exposes a `Walls` tab for the wall-mounted dev prototype, and the startup scene also spawns one real `StoreObject` instance of the wall prototype for Move/Delete smoke testing.
- Store coverage validation is sampled via `StoreArea::contains_polygon_sampled`.
- Camera clamp is viewport-aware and clamps by projected `WorldBounds`.
- Domain mutations are unified behind the `DomainCommand` system.
- Load/reset should clear UI/tool runtime state, including ribbon/build selection, not only gameplay sessions.
- Store overlays use a cache keyed by chunk coordinate.
- Highlight presentation keeps a small runtime state and writes colors only for dirty entities.

## Test Coverage

- Store chunk rules are covered by unit tests for initial ownership, adjacency, hole prevention, and purchase validation.
- Placement validation is covered by sampled coverage tests and collision rejection tests.
- Save/load is covered by an authority-restoration test.
- The overlay/highlight refactor is covered by unit tests for dirty entity collection, sprite color mapping, and available expansion chunk selection.
- Catalog validation is covered by tests for missing browse points and for factory mapping of capabilities into ECS components.
- Stage 5B.1 is covered by tests for boundary derivation, contiguous run generation, outer-corner anchoring, wall helpers, and the new wall-distance picking helper. Exterior content is still just a component/target foundation.

## Store Rules

- Initial store is 20 chunks total: `x = -5..-1`, `y = -4..-1`.
- The store anchor is the center of `WorldBounds`.
- Expansion is chunk-based only, using `4x4` store chunks.
- Freeform object placement stays separate from store chunk geometry.
- Store validation is one layer of gameplay rules, not a grid authority.

## Future Work

- Exact polygon coverage for store validation.
- NPC simulation (pathfinding and task planning reacting to `DomainEvents`).
- Economy system and currency-based validation in commands.
- Save migrations and multiple slots.
- Further decomposition of `DomainCommand` apply paths if new gameplay commands add more branching.
- Stage 5B.1 wall/exterior foundation and Stage 5B.2 wall-mounted preview integration.
