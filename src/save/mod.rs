pub mod extract;
pub mod io;
pub mod load;
pub mod types;
pub mod validation;

use crate::input::{InputAction, InputActionState};
use crate::objects::components::{StableObjectIdAllocator, StoreObject};
use crate::save::extract::extract_save_game;
use crate::save::io::{read_save_file, write_save_file_atomic};
use crate::save::load::{
    apply_load_plan, build_load_plan, clear_runtime_owned, reset_tool_runtime_flags,
    reset_tool_session, reset_ui_runtime,
};
use crate::save::validation::SaveLoadLimits;
use crate::store::{StoreArea, WorldBounds};
use crate::tools::ToolSessionState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy)]
pub struct QuickSaveRequested;

#[derive(Message, Debug, Clone, Copy)]
pub struct QuickLoadRequested;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<QuickSaveRequested>()
            .add_message::<QuickLoadRequested>()
            .init_resource::<SaveLoadLimits>()
            .add_systems(
                Update,
                (
                    save_load_hotkey_system,
                    handle_quick_save.in_set(crate::tools::ToolSet::Commit),
                    handle_quick_load.in_set(crate::tools::ToolSet::Commit),
                ),
            );
    }
}

fn save_load_hotkey_system(
    actions: Res<InputActionState>,
    mut quick_save: MessageWriter<QuickSaveRequested>,
    mut quick_load: MessageWriter<QuickLoadRequested>,
) {
    if actions.just_pressed(InputAction::QuickSave) {
        quick_save.write(QuickSaveRequested);
    }
    if actions.just_pressed(InputAction::QuickLoad) {
        quick_load.write(QuickLoadRequested);
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
struct QuickSaveParams<'w, 's> {
    events: MessageReader<'w, 's, QuickSaveRequested>,
    store: Res<'w, StoreArea>,
    allocator: Res<'w, StableObjectIdAllocator>,
    objects_query: Query<
        'w,
        's,
        (
            &'static crate::objects::components::ObjectStableId,
            &'static crate::objects::components::ObjectPrototypeId,
            &'static crate::objects::components::ObjectPlacementComponent,
        ),
        (With<StoreObject>, Without<crate::tools::ToolPreview>),
    >,
}

fn handle_quick_save(mut params: QuickSaveParams) {
    for _ in params.events.read() {
        let save = extract_save_game(&params.store, &params.allocator, &params.objects_query);
        match write_save_file_atomic("quicksave.json", &save) {
            Ok(_) => info!("QuickSave successful"),
            Err(e) => error!("QuickSave failed: {}", e),
        }
    }
}

#[derive(SystemParam)]
struct QuickLoadParams<'w, 's> {
    commands: Commands<'w, 's>,
    limits: Res<'w, SaveLoadLimits>,
    world_bounds: Res<'w, WorldBounds>,
    asset_server: Res<'w, AssetServer>,
    store: ResMut<'w, StoreArea>,
    allocator: ResMut<'w, StableObjectIdAllocator>,
    catalog: Res<'w, crate::objects::prototypes::ObjectCatalog>,
    existing_objects: Query<'w, 's, Entity, With<StoreObject>>,
    session: ResMut<'w, ToolSessionState>,
    return_state: ResMut<'w, crate::tools::ToolReturnState>,
    next_mode: ResMut<'w, NextState<crate::tools::ToolMode>>,
    selection: ResMut<'w, crate::tools::SelectionState>,
    build_selection: ResMut<'w, crate::objects::prototypes::BuildSelectionState>,
    ribbon_state: ResMut<'w, crate::ui::RibbonState>,
    ui_runtime: ResMut<'w, crate::ui::UiRuntime>,
    active_panel: ResMut<'w, crate::ui::ActiveInterfacePanel>,
    window_stack: ResMut<'w, crate::ui::UiWindowStack>,
    modal_stack: ResMut<'w, crate::ui::ModalStack>,
    targets: ResMut<'w, crate::input::PointerTargets>,
    cycle: ResMut<'w, crate::tools::PrimaryPointerCycle>,
    gate: ResMut<'w, crate::tools::ToolInputGate>,
    drag: ResMut<'w, crate::input::PointerDragState>,
    command_queue: ResMut<'w, crate::store::commands::DomainCommandQueue>,
    wall_cache: ResMut<'w, crate::store::WallVisualCache>,
    runtime_owned: Query<'w, 's, Entity, With<crate::objects::components::RuntimeOwned>>,
}

fn handle_quick_load(mut events: MessageReader<QuickLoadRequested>, mut p: QuickLoadParams) {
    if events.read().next().is_some() {
        match read_save_file("quicksave.json") {
            Ok(save) => match build_load_plan(save, p.limits.as_ref(), p.world_bounds.as_ref()) {
                Ok(plan) => {
                    info!(
                        "Applying load plan: {} chunks, {} objects",
                        plan.valid_chunks.len(),
                        plan.object_plans.len()
                    );

                    let runtime_owned_entities: Vec<_> = p.runtime_owned.iter().collect();

                    reset_tool_session(
                        &mut p.session,
                        &mut p.return_state,
                        &mut p.next_mode,
                        &mut p.selection,
                        &mut p.build_selection,
                    );
                    reset_tool_runtime_flags(
                        &mut p.cycle,
                        &mut p.gate,
                        &mut p.drag,
                        &mut p.command_queue,
                    );
                    reset_ui_runtime(
                        &mut p.ribbon_state,
                        &mut p.ui_runtime,
                        &mut p.active_panel,
                        &mut p.window_stack,
                        &mut p.modal_stack,
                        &mut p.targets,
                    );
                    clear_runtime_owned(&mut p.commands, runtime_owned_entities);
                    crate::store::clear_wall_cache(&mut p.wall_cache);

                    let report = apply_load_plan(
                        &mut p.commands,
                        &p.asset_server,
                        &mut p.store,
                        &mut p.allocator,
                        &p.catalog,
                        &p.existing_objects,
                        plan,
                    );

                    info!("QuickLoad successful: {:?}", report);
                }
                Err(e) => error!("Load validation failed: {:?}", e),
            },
            Err(e) => error!("QuickLoad failed: {}", e),
        }
    }
}
