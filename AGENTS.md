# AGENTS.md

This repository is a Rust + Bevy 2D isometric prototype with a strict ECS split:

- `InputActions -> ToolInputGate -> ToolRuntime`
- `UiRuntime -> UI requests/windows/modal stack`
- `Domain systems -> gameplay/state mutations`
- `Presentation -> overlays/highlights/transform sync`

## Codebase Map

Top-level entry points:

- [src/main.rs](/home/danilasar/data/projects/mallgame/src/main.rs) wires all plugins together and defines the schedule order.
- [src/input](/home/danilasar/data/projects/mallgame/src/input) owns raw input, pointer state, drag state, and cursor picking.
- [src/tools](/home/danilasar/data/projects/mallgame/src/tools) owns tool modes, tool runtime context, tool gates, and tool-specific systems.
- [src/ui](/home/danilasar/data/projects/mallgame/src/ui) owns UI runtime, modal stack, right dock, bottom build panel, camera controls, and world widgets.
- [src/store](/home/danilasar/data/projects/mallgame/src/store) owns world bounds, store chunks, expansion validation, and store overlays.
- [src/placement](/home/danilasar/data/projects/mallgame/src/placement) owns footprint geometry and placement validation.
- [src/presentation](/home/danilasar/data/projects/mallgame/src/presentation) owns isometric projection, transform sync, highlights, and footprint overlays.
- [src/objects](/home/danilasar/data/projects/mallgame/src/objects) owns gameplay/renderable object components, prototypes, and rotation data.

### Main Runtime Flow

The intended frame pipeline is:

1. `update_input_action_state`
2. `update_pointer_context`
3. `update_pointer_over_ui`
4. `update_hovered_object`
5. `update_tool_input_gate`
6. `modal_input_system` and other UI request systems
7. `camera_drag_system`
8. tool systems
9. placement validation
10. domain apply systems
11. presentation sync and overlays

The important boundary is that UI and tools only emit requests or mutate tool-local session state. Domain systems own the actual world mutation.

### Key Files

- [src/input/actions.rs](/home/danilasar/data/projects/mallgame/src/input/actions.rs) defines `InputAction`, bindings, and the snapshot resource used by the rest of the game.
- [src/input/pointer.rs](/home/danilasar/data/projects/mallgame/src/input/pointer.rs) converts screen cursor to world/projected coordinates.
- [src/input/camera_drag.rs](/home/danilasar/data/projects/mallgame/src/input/camera_drag.rs) keeps camera pan stable under zoom.
- [src/input/picking.rs](/home/danilasar/data/projects/mallgame/src/input/picking.rs) resolves hovered world entities.
- [src/tools/mode.rs](/home/danilasar/data/projects/mallgame/src/tools/mode.rs) defines `ToolMode`, hotkeys, and tool registration.
- [src/tools/context.rs](/home/danilasar/data/projects/mallgame/src/tools/context.rs) stores `ToolContext` and `ActiveToolAction`.
- [src/tools/gate.rs](/home/danilasar/data/projects/mallgame/src/tools/gate.rs) is the last guard before world interactions.
- [src/tools/build.rs](/home/danilasar/data/projects/mallgame/src/tools/build.rs) handles build ghost creation and build commits.
- [src/tools/move_tool.rs](/home/danilasar/data/projects/mallgame/src/tools/move_tool.rs) handles move selection, drag, commit, and rollback.
- [src/tools/delete.rs](/home/danilasar/data/projects/mallgame/src/tools/delete.rs) opens the delete modal and requests deletion.
- [src/tools/expansion.rs](/home/danilasar/data/projects/mallgame/src/tools/expansion.rs) handles store expansion selection and purchase modal requests.
- [src/ui/core.rs](/home/danilasar/data/projects/mallgame/src/ui/core.rs) defines UI layers, `BlocksWorldInput`, `UiRuntime`, and pointer-over-UI tracking.
- [src/ui/modal.rs](/home/danilasar/data/projects/mallgame/src/ui/modal.rs) owns the modal stack and confirm/cancel request handling.
- [src/ui/right_dock.rs](/home/danilasar/data/projects/mallgame/src/ui/right_dock.rs) renders the right-side interface switcher.
- [src/ui/build_panel.rs](/home/danilasar/data/projects/mallgame/src/ui/build_panel.rs) renders the bottom build panel and object catalog.
- [src/ui/camera_controls.rs](/home/danilasar/data/projects/mallgame/src/ui/camera_controls.rs) routes zoom/fullscreen requests.
- [src/ui/world_widgets.rs](/home/danilasar/data/projects/mallgame/src/ui/world_widgets.rs) renders contextual world widgets such as rotate.
- [src/store/area.rs](/home/danilasar/data/projects/mallgame/src/store/area.rs) defines store geometry, world bounds, and initial owned chunks.
- [src/store/chunks.rs](/home/danilasar/data/projects/mallgame/src/store/chunks.rs) defines chunk coordinates and hole detection helpers.
- [src/store/expansion.rs](/home/danilasar/data/projects/mallgame/src/store/expansion.rs) validates and applies store chunk purchases.
- [src/store/overlay.rs](/home/danilasar/data/projects/mallgame/src/store/overlay.rs) renders owned and candidate store chunk overlays.
- [src/store/validation.rs](/home/danilasar/data/projects/mallgame/src/store/validation.rs) validates footprint placement against owned store area.
- [src/placement/footprint.rs](/home/danilasar/data/projects/mallgame/src/placement/footprint.rs) converts local footprints into world polygons and tests overlaps.
- [src/presentation/projection.rs](/home/danilasar/data/projects/mallgame/src/presentation/projection.rs) converts between world and isometric projected space.
- [src/presentation/transform_sync.rs](/home/danilasar/data/projects/mallgame/src/presentation/transform_sync.rs) turns simulation state into render transforms and depth.
- [src/presentation/highlight.rs](/home/danilasar/data/projects/mallgame/src/presentation/highlight.rs) resolves highlight priorities and sprite tinting.
- [src/presentation/footprint_overlay.rs](/home/danilasar/data/projects/mallgame/src/presentation/footprint_overlay.rs) renders footprint outline overlays in world space.
- [src/objects/components.rs](/home/danilasar/data/projects/mallgame/src/objects/components.rs) contains the shared gameplay and presentation components.
- [src/objects/prototypes.rs](/home/danilasar/data/projects/mallgame/src/objects/prototypes.rs) maps build prototypes to assets, footprints, and rotation variants.
- [src/objects/rotation.rs](/home/danilasar/data/projects/mallgame/src/objects/rotation.rs) applies rotation requests by swapping sprite and footprint data.

### Core Data Model

- `WorldPos` is simulation position.
- `ProjectedPos` is the isometric projection result.
- `FootAnchor` marks the sorting and picking anchor.
- `Footprint` is local geometry for placement and collision.
- `StoreObject` marks objects that belong to the store and can be moved or deleted.
- `Rotatable` carries rotation variants, including sprite, footprint, foot anchor, and visual offset.
- `StoreArea` owns the bought store chunks.
- `WorldBounds` owns the outer world rectangle.
- `ToolContext` carries hovered entity, pointer coordinates, and active tool session.
- `ModalStack` owns modal lifecycle and blocking behavior.

### Events And Requests

These types are request boundaries, not direct world mutations:

- `ToolChangedRequested`
- `ObjectActionRequested`
- `StartMoveObjectRequested`
- `MoveObjectCommitted`
- `DeleteObjectRequested`
- `BuildObjectRequested`
- `SelectBuildObjectRequested`
- `RotateObjectRequested`
- `CameraControlRequested`
- `ModalRequest`
- `PurchaseStoreChunkRequested`

### Store Rules

- Initial store is 20 chunks total: `x = -5..-1`, `y = -4..-1`.
- The store anchor is the center of `WorldBounds`.
- Expansion is chunk-based only, using `4x4` store chunks.
- Freeform object placement stays separate from store chunk geometry.
- Store validation is one layer of the gameplay rules, not a grid authority.

## Project Rules

- Keep object placement freeform and continuous in `WorldPos`.
- Do not introduce tilemaps, logical grids, occupancy grids, or physics engines.
- Do not let UI mutate gameplay state directly. UI must emit requests/events.
- Preserve the existing `ToolRuntime` architecture.
- Keep store expansion as a domain/tool concern, not as a tile/build object.
- Keep camera bounds based on `WorldBounds`, not store bounds.
- Keep `StoreObject` as the marker for objects that tools may move/delete.

## World Model

- `WorldPos` is simulation space.
- Isometric projection is a presentation transform only.
- `ProjectedPos` and `Transform` are derived from simulation state.
- Footprints are local geometry that can later become polygon-based.
- Placement validation must check:
  - inside owned store area
  - no collision with blocking footprints
  - no self-collision for active move/build sessions

## UI Rules

- UI should use request/event types such as:
  - `ModalRequest`
  - `SelectBuildObjectRequested`
  - `CameraControlRequested`
  - `PurchaseStoreChunkRequested`
- UI blocking must be reflected in `PointerContext.over_ui` before `ToolInputGate`.
- Right dock, bottom build panel, modal stack, and world widgets are separate UI layers.
- UI overlays must not participate in picking.

## Store Rules

- Store area is made of owned `4x4` chunks.
- Initial store starts centered on `WorldBounds`.
- Current expansion policy allows only left and down by config, not by hardcoded geometry.
- Expansion validation must reject:
  - already owned chunks
  - non-adjacent chunks
  - direction-disallowed chunks
  - holes
  - chunks outside `WorldBounds`
- Objects are not snapped to chunks. Store chunks only gate placement and expansion.

## Code Style

- Prefer small ECS systems over large monolithic controllers.
- Keep pure validation code testable.
- Use `apply_patch` for manual edits.
- Default to ASCII unless the file already uses Unicode for a clear reason.
- Avoid unnecessary abstractions unless they match an existing pattern.

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

## Practical Editing Notes

- When touching tool behavior, check both tool state transitions and request emission.
- When touching UI, verify `BlocksWorldInput` and the order relative to `update_tool_input_gate`.
- When touching store validation, keep pure helpers testable and avoid mixing in UI or tool session state.
- When touching presentation, remember it must not mutate gameplay state.



Ниже — цельное описание проекта в текущем состоянии: что мы строим, зачем выбраны такие решения, как устроена архитектура, какие подсистемы уже есть, какие ограничения приняты и куда проект может развиваться дальше.

---

# Общее описание проекта

Проект — это **2D-изометрическая freeform-игра про магазин / торговое пространство**, вдохновлённая старыми социальными изометрическими играми и tycoon/decorator-играми вроде торговых кварталов, ферм, интерьеров и city-builder-сцен.

Ключевая идея проекта: игрок управляет магазином в изометрическом 2D-мире, свободно размещает объекты, перемещает их, удаляет, строит новые, расширяет игровую площадь магазина и со временем формирует собственную геометрию торгового пространства.

При этом проект принципиально **не tilemap-based** и **не grid-placement game**. Сетка существует, но она не используется как сетка привязки объектов. Объекты размещаются свободно, в continuous world coordinates.

Главная техническая цель — получить архитектуру, похожую на редактор игрового мира:

```text
freeform 2D world
+ isometric projection
+ ECS tools
+ UI runtime
+ modal/windows system
+ build catalog
+ expansion tool
+ placement validation
```

---

# Ключевые принципы проекта

## 1. Объекты размещаются свободно

Объекты имеют обычную `WorldPos(Vec2)` и могут находиться в любой continuous позиции:

```text
WorldPos { x: 142.37, y: -51.82 }
```

Они не привязываются:

* к клеткам;
* к тайлам;
* к expansion chunks;
* к визуальной сетке магазина.

Сетка и chunks ограничивают только область, где объект разрешено поставить.

---

## 2. Изометрия — это presentation layer

Мир логически остаётся 2D. Изометрия возникает на этапе визуализации:

```text
WorldPos
  → world_to_iso()
  → ProjectedPos
  → Transform
  → Sprite rendering
```

Игровая логика живёт в world space, а изометрическая проекция используется для отрисовки, depth sorting, overlay и cursor mapping.

---

## 3. Магазин — не tilemap

Магазин не является tilemap. Он представлен как **набор купленных 4×4 chunks**, которые определяют разрешённую область строительства.

```text
StoreArea = union of owned 4×4 chunks
```

Это даёт игроку свободу влиять на форму магазина: не просто покупать “20×20”, а выбирать конкретные секции площади на карте.

---

## 4. StoreGrid — не placement grid

Сетка магазина нужна для:

* визуального понимания площади;
* отображения купленных chunks;
* показа доступных expansion chunks;
* будущих зон, комнат, секций, декора пола;
* debug overlays.

Но она не является authority для placement.

---

## 5. UI не меняет gameplay напрямую

Любой UI-элемент отправляет request/event, а не мутирует мир сам.

Пример правильного flow:

```text
UI button
  → request/event
  → domain/tool system
  → gameplay state mutation
  → presentation update
```

Например:

```text
ConfirmPurchaseChunk modal
  → PurchaseStoreChunkRequested
  → validate_chunk_purchase
  → StoreArea.owned_chunks.insert(...)
```

---

# Технический стек

Проект построен на:

```text
Rust
Bevy
ECS
2D sprites
Camera2d
custom tools/runtime
custom store/expansion domain
custom UI/runtime
```

Bevy хорошо подходит под такую архитектуру, потому что tools, UI, placement, validation и domain logic удобно раскладываются на plugins, resources, components, events и systems. Bevy states дают finite-state-machine модель с `OnEnter`/`OnExit` schedules, а `NextState` используется для переключения состояний. Это важно для `ToolMode`. ([Docs.rs][1])

Для cursor/world mapping используется подход Bevy с `Camera::viewport_to_world_2d`, который как раз предназначен для получения 2D world position из координат viewport/cursor у 2D-камеры. ([Bevy][2])

Для input используется `ButtonInput`, который даёт `pressed`, `just_pressed` и `just_released`; `just_pressed` и `just_released` активны один кадр, поэтому подходят для action-based input вроде `Confirm`, `Cancel`, `PrimaryClick`. ([Docs.rs][3])

Для UI используются `Node` и `Interaction`. `Node` задаёт layout/style UI-элементов, а `Interaction` имеет состояния `Pressed`, `Hovered`, `None`, что важно для `BlocksWorldInput` и предотвращения протекания кликов из UI в игровой мир. ([Docs.rs][4])

---

# Координатные пространства

В проекте используются несколько пространств:

```text
World Space / Simulation Space
  → Isometric Projection
  → Projected Space
  → Camera View
  → Viewport
  → Screen/UI Space
```

## World Space

Это пространство игровой логики.

В нём находятся:

* `WorldPos`;
* footprints;
* StoreArea chunks;
* WorldBounds;
* placement validation;
* движение камеры;
* picking.

## Projected Space

Это результат изометрической проекции. Используется для:

* визуальной позиции спрайтов;
* depth sorting;
* world overlays;
* footprint outline;
* expansion overlays.

## Viewport / Screen Space

Это пространство UI, cursor position и camera output. UI-панели, модалки, кнопки и dock живут в screen/UI space.

---

# Основная архитектура

Текущая архитектура разделена на несколько runtime-слоёв:

```text
Input Runtime
  → raw input
  → InputActionState
  → ToolInputGate

Tool Runtime
  → ToolMode
  → Cursor / Move / Delete / Build / Expansion tools

UI Runtime
  → right dock
  → bottom build panel
  → modal stack
  → future windows

Store Runtime
  → WorldBounds
  → StoreArea
  → chunks
  → expansion validation
  → store overlays

Placement Runtime
  → footprint validation
  → blockers
  → owned chunk union check

Presentation Runtime
  → world_to_iso
  → transform sync
  → depth sorting
  → highlights
  → overlays
```

Это позволяет добавлять новые инструменты и UI-системы без переписывания input, picking, camera или placement.

---

# Текущая структура модулей

Сейчас проект примерно разложен так:

```text
src/
  main.rs

  input/
    mod.rs
    actions.rs
    pointer.rs
    camera_drag.rs
    picking.rs

  tools/
    mod.rs
    mode.rs
    context.rs
    gate.rs
    cursor.rs
    move_tool.rs
    delete.rs
    build.rs
    expansion.rs

  ui/
    mod.rs
    modal.rs
    build_panel.rs
    right_dock.rs
    camera_controls.rs
    world_widgets.rs

  store/
    mod.rs
    overlay.rs
    validation.rs
    expansion.rs
    chunks.rs
    area.rs

  placement/
    mod.rs
    validation.rs
    footprint.rs

  presentation/
    mod.rs
    projection.rs
    transform_sync.rs
    highlight.rs
    footprint_overlay.rs

  objects/
    mod.rs
    components.rs
    prototypes.rs
    rotation.rs
```

В реальном проекте файлы могут отличаться, но смысловая структура такая.

---

# Tool Runtime

В проекте есть несколько режимов управления:

```rust
enum ToolMode {
    Cursor,
    Move,
    Delete,
    Build,
    Expansion,
}
```

## Cursor Tool

Обычный режим.

Игрок:

* двигает камеру drag’ом;
* кликает по объектам;
* выполняет активные действия над интерактивными объектами.

Cursor tool не должен сам выполнять доменную логику объекта. Он отправляет intent/event, например:

```text
ObjectActionRequested
```

---

## Move Tool

Режим перемещения объектов.

Поведение:

* при hover над movable object подсвечивается footprint;
* click выбирает объект;
* объект начинает следовать за курсором без удержания кнопки;
* placement валидируется каждый кадр;
* invalid placement красит объект/preview в красный;
* повторный click commit’ит позицию, если valid;
* если invalid — объект возвращается на исходную позицию;
* Escape/right click отменяет перемещение.

Move tool работает только с:

```text
StoreObject + Movable
```

Это защищает фон, overlays, grid и world decor от случайного перемещения.

---

## Delete Tool

Режим удаления.

Поведение:

* hover над deletable object подсвечивает его красным;
* click открывает confirm modal;
* confirm отправляет `DeleteObjectRequested`;
* cancel закрывает modal.

Delete tool работает только с:

```text
StoreObject + Deletable
```

---

## Build Tool

Режим строительства объектов.

Он не активируется просто от кнопки “Строительство”. Сначала игрок открывает bottom build panel, затем выбирает объект в каталоге и нажимает “Установить”.

Flow:

```text
Object card Install
  → SelectBuildObjectRequested
  → BuildSession.selected_prototype = Some(...)
  → ToolMode::Build
  → spawn/update ghost
  → ghost follows cursor
  → placement validation
  → click valid = spawn real StoreObject
```

После успешного размещения build tool может оставаться активным с тем же prototype, чтобы игрок мог поставить несколько одинаковых объектов подряд.

---

## Expansion Tool

Новый инструмент расширения магазина.

Он выбирается из нижнего строительного меню. Когда active:

* нижняя панель сжимается;
* на ней показывается инструкция;
* на карте отображаются доступные expansion chunks;
* игрок выбирает конкретный 4×4 chunk;
* открывается confirm modal;
* confirm отправляет `PurchaseStoreChunkRequested`;
* domain system повторно валидирует покупку и добавляет chunk.

Expansion tool:

* не мутирует `StoreArea` напрямую;
* не создаёт UI напрямую;
* использует `PointerContext` и `ToolInputGate`;
* читает world position курсора;
* превращает cursor world position в `StoreChunkCoord`.

---

# ToolInputGate

`ToolInputGate` — центральный фильтр world input.

Он учитывает:

* pointer over UI;
* active modal;
* camera drag;
* click consumed after drag;
* cancel action;
* primary click;
* confirm action.

Задача gate — не дать tool-системам обрабатывать ввод, если input должен принадлежать UI/modal/camera.

Пример проблемы, которую он предотвращает:

```text
клик по карточке объекта
  не должен одновременно поставить объект в мире
```

или:

```text
клик по confirm modal
  не должен одновременно выбрать expansion chunk под модалкой
```

---

# Input Actions

Физические клавиши должны быть спрятаны за gameplay actions.

Пример:

```rust
enum InputAction {
    ToolCursor,
    ToolMove,
    ToolDelete,
    ToolBuild,
    PrimaryClick,
    SecondaryClick,
    Cancel,
    Confirm,
    CameraZoomIn,
    CameraZoomOut,
    ToggleFullscreen,
    PrintDebugPositions,
}
```

Преимущество:

* tools не знают про `Escape`, `Enter`, `Digit1`;
* UI и клавиатура вызывают одинаковые requests;
* позже можно добавить rebinding;
* можно поддержать gamepad/touch.

---

# UI Runtime

UI является отдельным слоем и не должен напрямую менять gameplay.

Сейчас есть:

```text
Right sidebar / interface switcher
Bottom build panel
Modal runtime
Camera controls
World/contextual widgets
```

## Right sidebar

Справа находится столбец переключателей интерфейсов.

Он не должен быть жёстко “tool panel”, потому что в будущем может открывать:

* build menu;
* inventory;
* object inspector;
* debug tools;
* settings;
* quests;
* economy panels.

---

## Bottom Build Panel

Нижняя строительная панель открывается через sidebar.

Она имеет два основных состояния:

### Objects mode

Панель раскрыта.

Показывает горизонтальный каталог объектов:

```text
card:
  name
  sprite
  Install button
```

Нажатие Install отправляет:

```text
SelectBuildObjectRequested { prototype }
```

### Expansion mode

Панель сжата.

Показывает краткую инструкцию:

```text
Выберите доступный блок 4×4 на карте
```

При этом active tool:

```text
ToolMode::Expansion
```

---

## Modal Runtime

Модалки универсальны и открываются через `ModalRequest`.

Текущие модалки:

```rust
enum ModalKind {
    ConfirmDelete { entity: Entity },
    ConfirmPurchaseChunk {
        coord: StoreChunkCoord,
        kind: StoreChunkKind,
    },
}
```

Modal runtime:

* показывает overlay;
* блокирует world input;
* обрабатывает Confirm/Cancel;
* отправляет domain events;
* сам не мутирует gameplay world напрямую.

---

# Store Runtime

Store runtime — новый доменный слой.

Он отвечает за:

* большой мир;
* площадь магазина;
* купленные chunks;
* проверку расширений;
* отображение expansion overlays;
* проверку placement внутри купленной площади.

---

## WorldBounds

`WorldBounds` — большой мир.

Он:

* всегда больше магазина;
* ограничивает камеру;
* не является bounds для placement объектов магазина;
* содержит магазин в центре.

```rust
#[derive(Resource)]
struct WorldBounds {
    rect: Rect,
}
```

Камера clamp’ится по `WorldBounds`, а не по магазину.

---

## StoreArea

Магазин представлен не прямоугольником, а набором купленных chunks.

```rust
#[derive(Resource)]
struct StoreArea {
    anchor: Vec2,
    cell_size: Vec2,
    chunk_size_cells: UVec2,
    owned_chunks: HashMap<StoreChunkCoord, StoreChunkData>,
    expansion_policy: StoreExpansionPolicy,
}
```

`anchor` — правый верхний угол стартового магазина.

Он находится в центре `WorldBounds`.

---

## StoreChunkCoord

Координата 4×4 chunk.

```rust
struct StoreChunkCoord {
    x: i32,
    y: i32,
}
```

Так как магазин стартует от правого верхнего угла и растёт влево/вниз:

```text
left  = x уменьшается
down  = y уменьшается
```

Стартовая область 20×16:

```text
x = -5..-1
y = -4..-1
```

То есть стартовые chunks:

```text
5 chunks по ширине
4 chunks по высоте
20 chunks total
```

---

## StoreChunkData и StoreChunkKind

Chunks уже типизированы на будущее.

```rust
enum StoreChunkKind {
    Default,
    // future:
    // Storage,
    // Premium,
    // Seasonal,
    // Utility,
}

struct StoreChunkData {
    kind: StoreChunkKind,
}
```

Сейчас все chunks могут быть `Default`, но в будущем можно добавить разные типы секций магазина.

---

## StoreExpansionPolicy

Политика расширения управляет тем, куда можно расширяться.

```rust
struct StoreExpansionPolicy {
    allow_left: bool,
    allow_right: bool,
    allow_up: bool,
    allow_down: bool,
    require_side_adjacency: bool,
    forbid_holes: bool,
}
```

Текущий config:

```text
allow_left = true
allow_down = true
allow_right = false
allow_up = false
require_side_adjacency = true
forbid_holes = true
```

Архитектурно расширение вправо/вверх возможно, но сейчас disabled policy.

---

# Expansion validation

Покупка chunk valid, если:

```text
1. Chunk ещё не куплен.
2. Chunk находится внутри WorldBounds.
3. Chunk смежен с owned chunk по стороне.
4. Direction разрешён текущей policy.
5. Покупка не создаёт дырку.
6. Будущие unlock/cost rules проходят.
```

Сейчас финансовой системы нет, поэтому цены не проверяются.

Типы причин invalid:

```rust
enum StoreChunkPurchaseInvalidReason {
    AlreadyOwned,
    OutsideWorldBounds,
    NotSideAdjacent,
    DirectionNotAllowed,
    WouldCreateHole,
    Locked,
    CannotAfford,
}
```

---

## Смежность

Разрешена только смежность по стороне:

```text
left
right
up
down
```

Диагональная смежность не считается.

---

## Запрет дырок

Дырки запрещены.

Для проверки используется flood fill:

```text
candidate_owned = owned_chunks + candidate

bounds = bounding box candidate_owned expanded by 1

flood fill снаружи по пустым chunks

если внутри bounds есть пустой chunk,
до которого flood fill не добрался,
значит образовалась дырка
```

Это чистая доменная функция и хороший кандидат для unit-тестов.

---

# Placement Runtime

Placement validation теперь проверяет:

```text
1. Footprint внутри WorldBounds.
2. Footprint внутри owned StoreArea chunks.
3. Footprint не пересекает blocking footprints.
4. Ghost/self игнорируется при moving/building.
```

Главная новая причина invalid:

```rust
PlacementInvalidReason::OutsideOwnedStoreArea
```

## Важное ограничение MVP

Если `contains_polygon` проверяет только вершины footprint, это допустимо для MVP, но не идеально. Большой footprint может пересечь некупленную область ребром, даже если вершины внутри купленных chunks.

Дырки запрещены, поэтому риск ниже, но позже стоит усилить проверку:

* sample points;
* midpoint edges;
* polygon vs union-of-chunks;
* AABB coverage.

---

# Object model

Объекты строятся из `BuildObjectCatalog`.

Прототип объекта:

```rust
struct BuildObjectPrototype {
    id: BuildObjectId,
    name: String,
    sprite: Handle<Image>,
    footprint: Footprint,
    foot_anchor: Vec2,
    visual_offset: Vec2,
    rotatable: Option<RotatablePrototype>,
}
```

UI-карточка хранит только `BuildObjectId`, а не копию всех данных.

После placement объект получает:

```text
StoreObject
WorldPos
Footprint
FootAnchor
VisualOffset
Sprite
Movable
Deletable
BlocksPlacement
```

Move/Delete работают только с `StoreObject`, чтобы не трогать:

* фон;
* grid;
* overlays;
* world decorations;
* ghosts;
* UI/world widgets.

---

# Rotation model

У объектов может быть поворот.

Не все объекты rotatable.

Поворот архитектурно хранится через variants:

```rust
struct Rotatable {
    current: usize,
    variants: Vec<RotationVariant>,
}

struct RotationVariant {
    sprite: Handle<Image>,
    footprint: Footprint,
    foot_anchor: Vec2,
    visual_offset: Vec2,
}
```

Даже если сейчас визуально просто меняется sprite, архитектурно variant должен хранить footprint и anchor, потому что повернутый объект может иметь другую геометрию placement.

---

# Presentation Runtime

Presentation отвечает за:

* изометрическую проекцию;
* sync `WorldPos → ProjectedPos → Transform`;
* depth sorting;
* highlights;
* footprint outline;
* store grid overlay;
* expansion overlay.

---

## Depth sorting

Объекты сортируются по foot anchor / projected y, а не по центру sprite.

Это важно для изометрических игр: высокие sprite могут иметь визуальный центр далеко от точки, которой они “стоят” на полу.

---

## Highlights

Highlights не должны напрямую задаваться tool-системами через `Sprite.color`.

Tool systems выставляют intent/state, а presentation layer превращает это в:

* tint;
* outline;
* footprint overlay;
* red invalid state;
* yellow selected perimeter.

Priority должен быть явным:

```text
DeleteDanger > Invalid > Valid > Selected > Hover
```

---

## Store overlay

Store overlay показывает:

* owned chunks;
* available expansion chunks;
* hovered available chunk.

Overlay не участвует в picking.

Для этого у overlay-сущностей должен быть marker вроде:

```text
NonInteractive
StoreChunkOverlay
```

---

# Система расширения магазина

Полный flow:

```text
1. Игрок открывает BuildPanel.
2. Нажимает "Расширение".
3. BuildPanel сжимается.
4. ToolMode::Expansion активируется.
5. На карте появляются доступные chunks.
6. Игрок hover’ит chunk.
7. Hovered chunk подсвечивается.
8. Click по valid chunk.
9. Открывается ConfirmPurchaseChunk modal.
10. Confirm.
11. Modal отправляет PurchaseStoreChunkRequested.
12. Domain system валидирует покупку повторно.
13. owned_chunks.insert(coord, StoreChunkData { kind }).
14. Overlay обновляется.
15. Chunk становится частью разрешённой площади магазина.
```

---

# UI blocking

Любые UI-элементы, которые должны блокировать world input, получают `BlocksWorldInput`.

Это касается:

* bottom build panel;
* object cards;
* install buttons;
* expansion compact panel;
* modal overlay;
* modal buttons;
* right sidebar;
* camera controls.

`PointerContext.over_ui` должен обновляться до `ToolInputGate`.

Если порядок нарушить, возможен баг:

```text
клик по UI
  → одновременно обрабатывается world tool
```

---

# System ordering

Желательный порядок:

```text
PreUpdate:
  update_input_action_state
  update_pointer_context
  update_pointer_over_ui
  update_hovered_object

Update:
  update_tool_input_gate
  modal_input_system
  ui button request systems
  camera_drag_system
  tool systems
  validate_active_placement
  apply domain events
  apply purchase chunk requests
  apply delete/build/move/rotate requests

PostUpdate:
  update_highlight_intents
  sync_visual_transform
  update_highlight_visuals
  update_store_grid_overlay
  update_expansion_overlay
  update_world_widgets
```

Ключевые правила:

```text
UI blocking до ToolInputGate.
Tool systems до validation.
Domain events до overlay rebuild.
Overlays после изменения StoreArea.
```

---

# Текущие реализованные подсистемы

По твоему последнему отчёту уже сделано:

```text
src/store/mod.rs
  WorldBounds
  StoreArea
  owned 4x4 chunks
  StoreExpansionPolicy
  validate_chunk_purchase
  would_create_hole flood fill
  placement validation внутри купленной площади

src/tools/expansion.rs
  ToolMode::Expansion
  ExpansionToolPlugin
  PointerContext + ToolInputGate
  valid click → ModalRequest::Open

src/ui/modal.rs
  ModalKind::ConfirmPurchaseChunk
  confirm → PurchaseStoreChunkRequested
  apply_purchase_store_chunk_requested

src/ui/build_panel.rs
  Objects mode: Chair / Table / Tree cards
  Expansion mode: compact instruction panel
  UI emits requests/events

src/placement/validation.rs
  footprint inside WorldBounds
  footprint inside owned store chunks
  blocking footprints check

Move/Delete
  only StoreObject

src/store/overlay.rs
  owned chunks contour
  available expansion chunks only in Expansion
  hovered valid chunk stronger highlight
  overlays ignored by picking

Camera
  clamped by WorldBounds, not StoreArea
```

Это соответствует целевой архитектуре.

---

# Принятые решения

## Не использовать tilemap

Причина: проект про freeform placement, а не про дискретную тайловую карту.

Tilemap навязал бы неправильную модель:

* объекты по клеткам;
* tile occupancy;
* grid-first мышление.

Здесь сетка — только визуальная/доменная область покупки площади.

---

## Не привязывать объекты к сетке

Причина: нужны попиксельные/continuous позиции, как в social/decorator играх.

Игрок должен чувствовать, что ставит мебель, деревья, витрины, столы свободно.

---

## Expansion не build object

Расширение магазина — это изменение разрешённой площади, а не объект на сцене. Поэтому оно живёт в `StoreExpansionPlugin` / `ExpansionTool`, а не в `BuildTool`.

---

## UI не мутирует world

Причина: события от UI, клавиатуры, tutorial, debug tools и будущих систем должны вести к одному и тому же domain flow.

---

## StoreArea не один Rect

Причина: магазин теперь может иметь произвольную форму из 4×4 chunks.

---

# Чего стоит избегать

Нельзя:

```text
1. Превращать магазин в tilemap.
2. Делать snap объектов к grid/chunks.
3. Использовать StoreGrid как authority placement.
4. Хранить StoreArea только как один Rect.
5. Делать expansion обычным build object.
6. Делать expansion фиксированными размерами 20×20 / 20×24.
7. Позволять UI напрямую менять StoreArea.
8. Позволять modal напрямую менять StoreArea.
9. Хранить prototype data в UI cards.
10. Давать overlays участвовать в picking.
11. Хардкодить left/down в geometry functions.
12. Полагаться только на OnEnter/OnExit для обновления выбранного prototype/tool state.
13. Разрешать diagonal-only adjacency.
14. Разрешать дырки.
15. Использовать WorldBounds как placement bounds.
16. Использовать StoreArea как camera bounds.
```

---

# Риски и технический долг

## 1. Coordinate edge cases

Нужно хорошо протестировать:

* anchor;
* отрицательные chunk coordinates;
* точки на границах chunks;
* `world_to_chunk_coord`;
* полуоткрытые интервалы `[min, max)`.

## 2. `contains_polygon`

MVP-реализация может быть слишком простой. Позже потребуется более точная проверка footprint внутри union owned chunks.

## 3. Overlay lifecycle

Нужно следить, чтобы overlays:

* не спавнились бесконечно каждый кадр;
* обновлялись после покупки chunk;
* удалялись при выходе из Expansion mode;
* не попадали в picking.

## 4. System ordering

Больше всего багов может возникнуть на стыке:

* UI;
* modal;
* ToolInputGate;
* ExpansionTool;
* Store overlay;
* domain events.

## 5. Save model

Сохранение должно опираться на stable IDs:

```text
owned_chunks
object prototype ids
object positions
object rotations
store chunk kinds
```

Не стоит сохранять runtime handles как authority.

---

# Тесты, которые стоит иметь

Pure unit tests:

```text
initial chunks count = 20
initial bounds x=-5..-1, y=-4..-1
world_to_chunk_coord near anchor
side_neighbors returns 4 only
diagonal adjacency invalid
already owned invalid
non-adjacent invalid
right/up invalid under current policy
left/down adjacent valid
would_create_hole detects simple hole
purchase applies valid chunk
contains_point true for owned area
contains_point false for unowned area
placement outside owned area invalid
```

Integration/manual tests:

```text
click bottom panel does not select chunk
click modal does not leak into world
switch Objects → Expansion cancels build ghost
switch Expansion → Objects clears expansion hover
confirm purchase adds chunk
cancel modal does not add chunk
Move cannot move object outside owned area
Delete cannot delete non-StoreObject
Camera remains clamped by WorldBounds
```

---

# Возможные будущие направления

## Разные типы chunks

```text
Default
Storage
Premium
StaffOnly
Utility
Seasonal
```

Каждый тип может иметь:

* свои правила placement;
* свои визуальные overlays;
* свои unlock conditions;
* свои бонусы.

## Финансовая система

Покупка chunk получит:

* цену;
* проверку баланса;
* стоимость по расстоянию от anchor;
* scaling cost по количеству купленных chunks.

## Unlock progression

Можно открыть:

* расширение вправо;
* расширение вверх;
* storage chunks;
* premium floor zones;
* новые вкладки build catalog.

## Более точная geometry validation

Можно перейти от vertex-only к:

* polygon clipping;
* occupancy sampling;
* union-of-rects coverage;
* navmesh-style allowed areas.

## NPC и pathing

Так как объекты freeform, NPC pathing лучше строить не на placement grid, а через:

* navmesh;
* continuous collision;
* graph over walkable regions;
* sampled navigation.

## Object inspector

Клик по объекту в Cursor mode может открывать окно:

* название;
* rotation;
* actions;
* delete;
* move;
* stats.

## Multi-selection / editor tools

Будущие tools:

* Rotate;
* Clone;
* Paint decor;
* Zone editor;
* Road/path editor;
* Floor brush;
* Debug overlays.

---

# Краткая формула проекта

Проект можно описать так:

```text
Это 2D-изометрический freeform store-builder на Bevy.

Игрок размещает объекты свободно, без snap-to-grid.
Магазин находится внутри большого мира.
Большой мир ограничивает камеру.
Магазин ограничивает строительство.
Площадь магазина состоит из купленных 4×4 chunks.
Расширение — отдельный инструмент выбора chunk на карте.
Покупка expansion chunk идёт через confirm modal.
UI работает через requests/events и не мутирует gameplay напрямую.
Tools используют общий ToolRuntime, PointerContext и ToolInputGate.
Placement validation проверяет footprint внутри owned chunks и blockers.
Presentation layer отвечает за изометрию, depth sorting, highlights и overlays.
```

Это хорошая основа для редактороподобного tycoon/decorator-проекта, где можно постепенно добавлять экономику, прогрессию, разные типы помещений, NPC, каталоги объектов и продвинутые инструменты редактирования.

[1]: https://docs.rs/bevy/latest/bevy/state/state/index.html?utm_source=chatgpt.com "bevy::state::state - Rust"
[2]: https://bevy.org/examples/2d-rendering/2d-viewport-to-world/?utm_source=chatgpt.com "2D Rendering / 2D Viewport To World"
[3]: https://docs.rs/bevy/latest/bevy/input/prelude/struct.ButtonInput.html?utm_source=chatgpt.com "ButtonInput in bevy::input::prelude - Rust"
[4]: https://docs.rs/bevy/latest/bevy/ui/struct.Node.html?utm_source=chatgpt.com "Node in bevy::ui - Rust"
