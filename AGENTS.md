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
- The Ribbon is the active build selection surface; `BuildSelectionState` is the tool-config source of truth for the chosen prototype.
- The old bottom build panel terminology is obsolete; build selection now lives in the Ribbon and the inspector/right dock surfaces.
- The right dock is interface switching UI and the inspector surface, not gameplay logic.
- Load/reset code must clear runtime UI/tool state as well as gameplay sessions; do not leave Ribbon or selection state behind after restore.

## Documentation

- Read [docs/README.md](/home/danilasar/data/projects/mallgame/docs/README.md) for the docs entry point.
- Read [docs/architecture.md](/home/danilasar/data/projects/mallgame/docs/architecture.md) for the detailed codebase map and runtime model.
- Read [docs/runtime-notes.md](/home/danilasar/data/projects/mallgame/docs/runtime-notes.md) for current runtime hot paths, tests, and quality notes.
- Read [docs/catalog-stage5a.md](/home/danilasar/data/projects/mallgame/docs/catalog-stage5a.md) for the Stage 5A prototype/catalog/capability model.
- Read [docs/walls-stage5b.md](/home/danilasar/data/projects/mallgame/docs/walls-stage5b.md) for the Stage 5B.1 boundary wall and exterior layer model.
- Read [docs/wall-mounted-stage5b3.md](/home/danilasar/data/projects/mallgame/docs/wall-mounted-stage5b3.md) for the current wall-mounted object MVP.
- Read [docs/wall-mounted-stage5b4.md](/home/danilasar/data/projects/mallgame/docs/wall-mounted-stage5b4.md) for generic wall-mounted move.
- Read [docs/doorway-stage5b5.md](/home/danilasar/data/projects/mallgame/docs/doorway-stage5b5.md) for doorway/access-zone hardening.

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
