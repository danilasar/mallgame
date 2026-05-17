# AGENTS.md

This repository is a Rust + Bevy 2D isometric prototype with a strict ECS split:

- `InputActions -> ToolInputGate -> ToolRuntime`
- `UiRuntime -> UI requests/windows/modal stack`
- `Domain systems -> gameplay/state mutations`
- `Presentation -> overlays/highlights/transform sync`

## Working Rules

- Keep object placement freeform and continuous in `WorldPos`.
- Do not introduce tilemaps, logical grids, occupancy grids, or physics engines.
- Do not let UI mutate gameplay state directly. UI must emit requests/events.
- Preserve the existing `ToolRuntime` architecture.
- Keep store expansion as a domain/tool concern, not as a tile/build object.
- Keep camera bounds based on `WorldBounds`, not store bounds.
- Keep `StoreObject` as the marker for objects that tools may move/delete.
- Prefer request/event boundaries over direct resource mutation when a change crosses from UI into tool or domain state.

## Current Architecture Summary

- `ToolMode` controls the active world interaction mode.
- `ToolSessionState` owns unfinished build/move/expansion sessions.
- `PointerTargets` and `PrimaryPointerCycle` are the current pointer ownership layer.
- `ToolPreview`, `PlacementPreview`, `PreviewSource`, `RuntimeOwned`, and `InteractionRole` are real transient/runtime components.
- `StoreArea` owns bought `4x4` chunks; `WorldBounds` owns the outer world.
- `StoreArea::contains_polygon_sampled` is sampled coverage, not exact polygon clipping.
- The bottom build panel is UI surface; it must emit requests instead of mutating gameplay state directly.
- The right dock is interface switching UI, not gameplay logic.

## Documentation

- Read [docs/README.md](/home/danilasar/data/projects/mallgame/docs/README.md) for the docs entry point.
- Read [docs/architecture.md](/home/danilasar/data/projects/mallgame/docs/architecture.md) for the detailed codebase map and runtime model.

## Verification

- Run `cargo check` after structural changes.
- Run `cargo test` for changes in store validation, placement, input, or tool lifecycles.
- When a change affects UI or camera behavior, verify the runtime path manually if possible.

## What Not To Do

- Do not revert unrelated user changes.
- Do not hardcode keyboard keys inside gameplay systems when an input action layer exists.
- Do not add hidden coupling between UI and world mutation.
- Do not make overlays selectable or interactive.
- Do not make store expansion a normal object prototype.

