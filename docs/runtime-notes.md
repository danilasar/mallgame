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

## Remaining Technical Debt

- `StoreArea::contains_polygon_sampled` is still sampled coverage, not exact polygon clipping.
- `apply_domain_commands` will grow if new gameplay commands are added without further decomposition.
- Camera clamping is pragmatic viewport-aware logic, not an exact geometry solver.

## When Extending The Runtime

- If a system touches every entity each frame, ask whether it can become delta-driven or cache-backed.
- If a system needs to mutate gameplay truth, route it through request/command boundaries.
- If a runtime entity is transient, keep it under the transient ownership model and test cleanup explicitly.

