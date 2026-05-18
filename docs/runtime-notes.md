# Runtime Notes

This file captures the current runtime-quality state of the codebase. It is meant to stay narrower than `architecture.md`: not a full map, but the things that matter when touching hot paths or adding systems.

## Current Hot Paths

- `StoreOverlay` is no longer a full rebuild system. It caches owned/available chunk overlay entities and only refreshes the hovered expansion chunk and visibility transitions.
- `Highlight` is no longer a full scan over all `Interactive` entities. It tracks a small runtime state and only rewrites `HighlightIntent` and sprite tint for dirty entities.
- `apply_domain_commands` is still the largest domain dispatcher. It is typed and clippy-clean, but it remains the main maintainability hotspot if more gameplay commands are added.

## What Is Tested

- Store chunk ownership and expansion rules.
- Store hole prevention and direction policy.
- Placement validation and sampled store coverage.
- Save/load restoration of gameplay authority.
- Highlight dirty-entity collection and tint selection.
- Expansion overlay frontier selection.

## What To Preserve

- Freeform `WorldPos` remains the gameplay authority.
- UI emits requests/events; it does not mutate gameplay state directly.
- Tool sessions stay preview-based.
- Overlays and highlight visuals stay runtime-owned and transient.
- Store expansion remains chunk-based and separate from object placement.
- Exterior is currently a component/target foundation inside `WorldBounds`, not an authored content layer yet.
- Derived store walls are cache/delta-synced, start at the outer top-right owned chunk, extend only through contiguous boundary runs, and must not become save authority.
- Stage 5B.2 wall-mounted placement is preview-only: wall prototypes use `PointerTargets.wall_surface` and `WallAttachmentPoint`, but clicks do not create real build commands yet.
- The Ribbon now includes a `Walls` tab and the startup scene spawns one real `StoreObject` instance of the wall prototype so Move/Delete can be smoke-tested without entering tools.

## Remaining Technical Debt

- `StoreArea::contains_polygon_sampled` is still sampled coverage, not exact polygon clipping.
- `apply_domain_commands` will grow if new gameplay commands are added without further decomposition.
- Camera clamping is pragmatic viewport-aware logic, not an exact geometry solver.
- Stage 5B.1 must keep picking separation explicit: `WorldObject`, `WallSurface`, and `Exterior` are distinct interaction domains.
- Stage 5B.2 wall-mounted previews remain transient; save/load must not pick them up.

## When Extending The Runtime

- If a system touches every entity each frame, ask whether it can become delta-driven or cache-backed.
- If a system needs to mutate gameplay truth, route it through request/command boundaries.
- If a runtime entity is transient, keep it under the transient ownership model and test cleanup explicitly.
