# Architecture

This project is a Rust + Bevy 2D isometric prototype for a freeform store builder/editor-like game.

## Runtime Layers

- `InputActions` snapshot physical keyboard/mouse input into gameplay actions.
- `ToolInputGate` converts input, pointer ownership, and modal/UI blocking into safe tool signals.
- `ToolRuntime` owns active tool mode and unfinished tool sessions.
- `UiRuntime` owns right dock, bottom build panel, modal stack, camera controls, and world widgets.
- `Domain systems` mutate gameplay truth.
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
9. placement validation
10. domain apply systems
11. presentation sync and overlays

## Module Map

- `src/main.rs`: app setup and plugin order.
- `src/input`: raw input, pointer conversion, drag state, and picking.
- `src/tools`: tool modes, pointer gate, sessions, previews, and tool-specific systems.
- `src/ui`: UI layers, modals, right dock, build panel, camera controls, and world widgets.
- `src/store`: world bounds, store chunks, expansion validation, and store overlays.
- `src/placement`: footprint geometry and placement validation.
- `src/presentation`: projection, transform sync, highlights, and footprint overlays.
- `src/objects`: shared components, prototypes, and rotation data.

## Key Runtime Types

- `WorldPos` is simulation position.
- `ProjectedPos` is isometric projection result.
- `FootAnchor` marks sorting/picking anchor.
- `Footprint` is local collision/placement geometry.
- `StoreObject` marks entities that live inside the store and can be moved or deleted.
- `Rotatable` carries rotation variants for sprite, footprint, foot anchor, and visual offset.
- `StoreArea` owns bought `4x4` chunks.
- `WorldBounds` owns the outer world rectangle.
- `ToolContext` stores hovered entity, pointer coordinates, and active session data.
- `ToolSessionState` stores current build/move/expansion sessions.
- `ModalStack` stores modal lifecycle and blocking state.
- `PointerTargets` stores hovered world object/widget/debug target separation.
- `PrimaryPointerCycle` stores primary click ownership across frame boundaries.

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

## Current Implementation Notes

- Tool sessions are preview-based for build and move.
- The build panel currently switches prototypes by request and may restart the build session in place when already in `Build`.
- Store coverage validation is sampled via `StoreArea::contains_polygon_sampled`.
- Camera clamp is viewport-aware and clamps by projected `WorldBounds`.
- `SelectionState` and a real `ObjectInspector` are still future work.

## Store Rules

- Initial store is 20 chunks total: `x = -5..-1`, `y = -4..-1`.
- The store anchor is the center of `WorldBounds`.
- Expansion is chunk-based only, using `4x4` store chunks.
- Freeform object placement stays separate from store chunk geometry.
- Store validation is one layer of gameplay rules, not a grid authority.

## Future Work

- Selection/inspector state.
- Exact polygon coverage for store validation.
- More request-driven UI surfaces.
- Save/load of gameplay truth only, not runtime previews or overlays.

