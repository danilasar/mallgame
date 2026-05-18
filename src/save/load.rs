use bevy::prelude::*;
use std::collections::HashSet;
use crate::save::types::*;
use crate::save::validation::*;
use crate::store::{StoreArea, StoreChunkData, WorldBounds};
use crate::objects::components::{StoreObject, StableObjectIdAllocator};
use crate::objects::prototypes::{spawn_store_object_from_prototype, SpawnStoreObjectParams, BuildPrototypes};
use crate::tools::{ToolSessionState, ToolReturnState, ToolMode, PrimaryPointerCycle, ToolInputGate};
use crate::input::{PointerTargets, PointerDragState};
use crate::ui::ModalStack;

#[allow(dead_code)]
pub struct LoadPlan {
    pub save: SaveGame,
    pub normalized_next_object_id: u64,
    pub valid_chunks: Vec<StoreChunkSave>,
    pub object_plans: Vec<ObjectLoadPlan>,
    pub report: LoadReport,
}

pub enum ObjectLoadPlan {
    Spawn(ObjectSave),
    #[allow(dead_code)]
    Skip { id: crate::objects::components::StableObjectId, reason: LoadIssue },
}

pub fn build_load_plan(
    save: SaveGame, 
    limits: &SaveLoadLimits,
    world_bounds: &WorldBounds,
) -> Result<LoadPlan, SaveLoadError> {
    if save.version != CURRENT_SAVE_VERSION {
        return Err(SaveLoadError::UnsupportedVersion(save.version));
    }

    if save.objects.len() > limits.max_objects || save.store.owned_chunks.len() > limits.max_chunks {
        return Err(SaveLoadError::FatalValidationError(vec![])); 
    }

    let store_report = validate_loaded_store_area(&save.store, world_bounds);
    if store_report.fatal {
        return Err(SaveLoadError::FatalValidationError(store_report.issues));
    }

    let mut issues = store_report.issues;
    let mut seen_ids = HashSet::new();
    let mut max_loaded_id = 0u64;
    let mut object_plans = Vec::new();

    for obj in &save.objects {
        if !seen_ids.insert(obj.id.0) {
            return Err(SaveLoadError::FatalValidationError(vec![LoadIssue::DuplicateStableObjectId(obj.id)]));
        }
        max_loaded_id = max_loaded_id.max(obj.id.0);

        if !obj.world_pos.x.is_finite() || !obj.world_pos.y.is_finite() {
            issues.push(LoadIssue::NonFiniteWorldPos { object_id: obj.id });
            object_plans.push(ObjectLoadPlan::Skip { id: obj.id, reason: LoadIssue::NonFiniteWorldPos { object_id: obj.id } });
            continue;
        }

        object_plans.push(ObjectLoadPlan::Spawn(obj.clone()));
    }

    let normalized_next_object_id = save.next_object_id.max(max_loaded_id + 1);
    if normalized_next_object_id > save.next_object_id {
        issues.push(LoadIssue::AllocatorNextIdTooSmall { 
            save_next: save.next_object_id, 
            normalized_next: normalized_next_object_id 
        });
    }

    Ok(LoadPlan {
        save: save.clone(),
        normalized_next_object_id,
        valid_chunks: store_report.valid_chunks,
        object_plans,
        report: LoadReport {
            loaded_objects: 0, 
            skipped_objects: issues.len(),
            loaded_chunks: save.store.owned_chunks.len(),
            issues,
        },
    })
}

pub fn reset_runtime_for_load(
    commands: &mut Commands,
    session: &mut ResMut<ToolSessionState>,
    return_state: &mut ResMut<ToolReturnState>,
    next_mode: &mut ResMut<NextState<ToolMode>>,
    selection: &mut ResMut<crate::tools::SelectionState>,
    modal_stack: &mut ResMut<ModalStack>,
    targets: &mut ResMut<PointerTargets>,
    cycle: &mut ResMut<PrimaryPointerCycle>,
    mut gate: ResMut<ToolInputGate>,
    mut drag: ResMut<PointerDragState>,
    mut command_queue: ResMut<crate::store::commands::DomainCommandQueue>,
    runtime_owned: &Query<Entity, With<crate::objects::components::RuntimeOwned>>,
    ) {
    session.active = None;
    return_state.previous = None;
    next_mode.set(ToolMode::Cursor);
    selection.primary = None;
    modal_stack.stack.clear();
    targets.world_object = None;
    targets.world_widget = None;
    targets.debug = None;
    cycle.owner = crate::tools::PointerPressOwner::None;
    gate.world_blocked = false;
    drag.is_camera_dragging = false;
    command_queue.commands.clear();

    for entity in runtime_owned {
        commands.entity(entity).despawn();
    }
    }


pub fn apply_load_plan(
    commands: &mut Commands,
    asset_server: &AssetServer,
    store: &mut ResMut<StoreArea>,
    allocator: &mut ResMut<StableObjectIdAllocator>,
    _prototypes: &BuildPrototypes,
    existing_objects: &Query<Entity, With<StoreObject>>,
    plan: LoadPlan,
) -> LoadReport {
    // 1. Clear existing objects
    for entity in existing_objects {
        commands.entity(entity).despawn();
    }

    // 2. Rebuild StoreArea
    store.owned_chunks.clear();
    for chunk in &plan.valid_chunks {
        store.owned_chunks.insert(chunk.coord, StoreChunkData { kind: chunk.kind });
    }

    // 3. Spawn objects
    let mut loaded_count = 0;
    for object_plan in &plan.object_plans {
        if let ObjectLoadPlan::Spawn(obj) = object_plan {
            spawn_store_object_from_prototype(
                commands,
                asset_server,
                SpawnStoreObjectParams {
                    stable_id: obj.id,
                    prototype_id: obj.prototype_id,
                    world_pos: Vec2::new(obj.world_pos.x, obj.world_pos.y),
                    rotation_index: obj.rotation_index,
                },
            );
            loaded_count += 1;
        }
    }

    // 4. Update allocator
    allocator.next = plan.normalized_next_object_id;

    let mut report = plan.report;
    report.loaded_objects = loaded_count;
    report
}
