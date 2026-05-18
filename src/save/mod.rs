pub mod types;
pub mod extract;
pub mod load;
pub mod io;
pub mod validation;

use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::store::{StoreArea, WorldBounds};
use crate::objects::components::{StoreObject, StableObjectIdAllocator};
use crate::tools::ToolSessionState;
use crate::save::validation::SaveLoadLimits;
use crate::save::extract::extract_save_game;
use crate::save::load::{build_load_plan, apply_load_plan, reset_runtime_for_load};
use crate::save::io::{write_save_file_atomic, read_save_file};
use crate::input::{InputAction, InputActionState};

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
            .add_systems(Update, (
                save_load_hotkey_system,
                handle_quick_save.in_set(crate::tools::ToolSet::Commit),
                handle_quick_load.in_set(crate::tools::ToolSet::Commit),
            ));
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

fn handle_quick_save(
    mut events: MessageReader<QuickSaveRequested>,
    store: Res<StoreArea>,
    allocator: Res<StableObjectIdAllocator>,
    objects_query: Query<(
        &crate::objects::components::ObjectStableId,
        &crate::objects::components::ObjectPrototypeId,
        &crate::objects::components::WorldPos,
        Option<&crate::objects::rotation::Rotatable>,
    ), (
        With<StoreObject>,
        Without<crate::tools::ToolPreview>,
    )>,
) {
    for _ in events.read() {
        let save = extract_save_game(&store, &allocator, &objects_query);
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
    prototypes: Res<'w, crate::objects::prototypes::BuildPrototypes>,
    existing_objects: Query<'w, 's, Entity, With<StoreObject>>,
    session: ResMut<'w, ToolSessionState>,
    return_state: ResMut<'w, crate::tools::ToolReturnState>,
    next_mode: ResMut<'w, NextState<crate::tools::ToolMode>>,
    selection: ResMut<'w, crate::tools::SelectionState>,
    modal_stack: ResMut<'w, crate::ui::ModalStack>,
    targets: ResMut<'w, crate::input::PointerTargets>,
    cycle: ResMut<'w, crate::tools::PrimaryPointerCycle>,
    gate: ResMut<'w, crate::tools::ToolInputGate>,
    drag: ResMut<'w, crate::input::PointerDragState>,
    runtime_owned: Query<'w, 's, Entity, With<crate::objects::components::RuntimeOwned>>,
}

fn handle_quick_load(
    mut events: MessageReader<QuickLoadRequested>,
    mut p: QuickLoadParams,
) {
    for _ in events.read() {
        match read_save_file("quicksave.json") {
            Ok(save) => {
                match build_load_plan(save, p.limits.as_ref(), p.world_bounds.as_ref()) {
                    Ok(plan) => {
                        info!("Applying load plan: {} chunks, {} objects", plan.valid_chunks.len(), plan.object_plans.len());
                        
                        reset_runtime_for_load(
                            &mut p.commands,
                            &mut p.session,
                            &mut p.return_state,
                            &mut p.next_mode,
                            &mut p.selection,
                            &mut p.modal_stack,
                            &mut p.targets,
                            &mut p.cycle,
                            &mut p.gate,
                            &mut p.drag,
                            &p.runtime_owned,
                        );
                        
                        let report = apply_load_plan(
                            &mut p.commands,
                            &p.asset_server,
                            &mut p.store,
                            &mut p.allocator,
                            &p.prototypes,
                            &p.existing_objects,
                            plan,
                        );
                        
                        info!("QuickLoad successful: {:?}", report);
                    }
                    Err(e) => error!("Load validation failed: {:?}", e),
                }
            }
            Err(e) => error!("QuickLoad failed: {}", e),
        }
        break; 
    }
}
