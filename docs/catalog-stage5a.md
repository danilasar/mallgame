# Stage 5A Catalog And Capabilities

This file documents the concrete catalog/capability layer that Stage 5A added. It is intentionally narrower than `architecture.md`: this is the part that matters when adding or validating prototypes.

## ObjectPrototype v2

`ObjectPrototype` is the data bundle used by the Ribbon, the spawn factory, and save/load compatibility layers.

It currently contains:

- `id: BuildObjectId`
- `display: ObjectDisplaySpec`
- `catalog: ObjectCatalogSpec`
- `placement: PlacementSpec`
- `visuals: VisualSpec`
- `rotation: RotationSpec`
- `capabilities: Vec<ObjectCapabilitySpec>`
- `initial_state: ObjectInitialStateSpec`

## ObjectDisplaySpec

Ribbon-facing display metadata:

- `display_name`
- `description: Option<String>`
- `icon: Option<String>`

The current runtime requires a non-empty `display_name`. `description` and `icon` are optional; the icon is still a placeholder path, not a fully wired asset pipeline.

## ObjectCatalogSpec

Ribbon organization metadata:

- `category: ObjectCategory`
- `ribbon_tab: BuildRibbonTab`
- `ribbon_group: BuildRibbonGroup`
- `sort_order: i32`
- `availability: CatalogAvailability`

This is not gameplay truth. It only drives how prototypes appear in the Ribbon.

## PlacementSpec

Stage 5A started floor-only placement:

- `kind: PlacementKind::Floor`
- `footprint_half_extents: Vec2`
- `placement_blocker: bool`
- `navigation_blocker: bool`

`placement_blocker` becomes the `BlocksPlacement` ECS component on the spawned entity.

Stage 5B.3 extends this with `PlacementKind::WallMounted` for the wall-mounted MVP. Wall-mounted prototypes still live in the same catalog, but their real placement authority is `ObjectPlacement::WallMounted`, not floor `WorldPos`.

## VisualSpec

Rendering/visual identity fields:

- `asset_path`
- `asset_id`
- `sprite_size`
- `foot_anchor`
- `sort_bias`

The factory uses these values to spawn the real runtime entity. `asset_id` is the persistent visual label used in logs and save-friendly references.

## RotationSpec

Rotation is variant-based, not sprite-rotation-based:

- `kind: RotationKind::None | TwoVariants`
- `rotated_asset_path: Option<String>`

For `TwoVariants`, the factory creates a second `RotationVariant` with the rotated sprite and swapped footprint dimensions. This keeps future geometry changes open without treating rotation as a pure texture transform.

## ObjectCapabilitySpec

Stage 5A capabilities are mapped into typed ECS components on spawn:

- `ProductContainer(ProductContainerSpec)`
- `CheckoutPoint(CheckoutPointSpec)`
- `Decor(DecorSpec)`
- `NpcInteractionPoints(NpcInteractionPointsSpec)`
- `WallMounted(WallMountedSpec)`
- `Window(WindowSpec)`

### ProductContainer

Spawns a `ProductContainer` component with:

- `kind: ProductContainerKind::Shelf | Rack | Fridge`
- `capacity_class: ContainerCapacityClass::Small | Medium | Large`

This is the capability behind shelving/rack/fridge-style objects.

### CheckoutPoint

Spawns a `CheckoutPoint` component with:

- `kind: CheckoutKind::BasicRegister`

This is the capability behind checkout/cashier-style objects.

### Decor

Spawns a `Decor` component with:

- `kind: DecorKind::Plant | Sign | Misc`

This is a low-interaction decorative capability, not a gameplay container.

### NpcInteractionPoints

Spawns a runtime `NpcInteractionPoints` component containing a list of points:

- `local_pos`
- `facing`
- `kind: NpcInteractionKind::BrowseProducts | Checkout`

This is a placeholder affordance layer for future NPC behavior. It is not NPC simulation.

### WallMounted

Spawns a runtime `WallMounted` component for `PlacementKind::WallMounted` prototypes.

The spec contains:

- `width`
- `height`
- `allowed_sides`
- `default_height_on_wall`

This is the capability behind wall decor/poster-style objects. It does not imply doors, wall cutouts, navigation portals, or window transparency.

### Window

Spawns a runtime `WallWindow` component with:

- `width`
- `height`
- `glass_alpha`

This is visual-only window semantics. It does not cut wall geometry, affect collision, or create navigation portals.

## Current Sample Prototypes

The startup catalog currently defines:

- `fixture.shelf.basic`
- `service.checkout.basic`
- `decor.plant.tree`
- `wall.decor.placeholder`
- `wall.window.basic_visual`

For save/load compatibility, there are alias entries:

- `chair` → shelf prototype
- `table` → checkout prototype
- `tree` → tree prototype

## Catalog Validation

`validate_object_catalog` currently checks:

- `display.display_name` must not be empty
- every `ProductContainer` prototype must expose at least one `BrowseProducts` interaction point
- every `CheckoutPoint` prototype must expose at least one `Checkout` interaction point

The validation result is logged at startup. There is also a unit test for the missing browse-point case.

## Runtime Mapping

The factory currently turns prototype data into runtime ECS components:

- `PlacementSpec` → floor `Footprint` and optional `BlocksPlacement` for floor objects only
- `WallMountedSpec` → `WallMounted` and `WallMountedBounds` for wall-mounted objects
- `VisualSpec` → `Sprite`, `WorldPos`, `FootAnchor`, `VisualOffset`, `SortLayer`, `SortBias`
- `ObjectCapabilitySpec` → `ProductContainer`, `CheckoutPoint`, `Decor`, `NpcInteractionPoints`, `WallMounted`, `Window`
- `ObjectPrototype.id` → `ObjectPrototypeId`
- `VisualSpec.asset_id` → entity `Name`

## Notes

- Ribbon metadata is stage 5A UI metadata, not gameplay truth.
- Capability components are what runtime systems should query.
- The catalog is still intentionally small; the shape is what matters, not the number of sample objects.
