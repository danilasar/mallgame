use crate::input::{PointerDragState, PointerTargets};
use crate::objects::components::{
    ObjectPlacement, StableObjectIdAllocator, StoreObject, WallAttachmentPoint,
};
use crate::objects::prototypes::{
    BuildRibbonTab, BuildSelectionState, ObjectCatalog, SpawnStoreObjectParams,
    spawn_store_object_from_prototype, wall_opening_spec,
};
use crate::save::types::*;
use crate::save::validation::*;
use crate::store::boundary::DirtyWallOpeningSegments;
use crate::store::{StoreArea, StoreChunkData, WallSegmentKey, WorldBounds};
use crate::tools::{
    PrimaryPointerCycle, ToolInputGate, ToolMode, ToolReturnState, ToolSessionState,
};
use crate::ui::{ActiveInterfacePanel, ModalStack, RibbonState, UiRuntime, UiWindowStack};
use bevy::prelude::*;
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Resource, Clone)]
pub struct LoadPlan {
    pub save: SaveGame,
    pub normalized_next_object_id: u64,
    pub valid_chunks: Vec<StoreChunkSave>,
    pub object_plans: Vec<ObjectLoadPlan>,
    pub report: LoadReport,
}

#[derive(Clone)]
pub enum ObjectLoadPlan {
    Spawn(ObjectSave),
    #[allow(dead_code)]
    Skip {
        id: crate::objects::components::StableObjectId,
        reason: LoadIssue,
    },
}

pub fn build_load_plan(
    save: SaveGame,
    limits: &SaveLoadLimits,
    world_bounds: &WorldBounds,
) -> Result<LoadPlan, SaveLoadError> {
    if save.version != CURRENT_SAVE_VERSION {
        return Err(SaveLoadError::UnsupportedVersion(save.version));
    }

    if save.objects.len() > limits.max_objects || save.store.owned_chunks.len() > limits.max_chunks
    {
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
            return Err(SaveLoadError::FatalValidationError(vec![
                LoadIssue::DuplicateStableObjectId(obj.id),
            ]));
        }
        max_loaded_id = max_loaded_id.max(obj.id.0);

        match &obj.placement {
            ObjectPlacementSave::Floor { world_pos, .. } => {
                if !world_pos.x.is_finite() || !world_pos.y.is_finite() {
                    issues.push(LoadIssue::NonFiniteWorldPos { object_id: obj.id });
                    object_plans.push(ObjectLoadPlan::Skip {
                        id: obj.id,
                        reason: LoadIssue::NonFiniteWorldPos { object_id: obj.id },
                    });
                    continue;
                }
            }
            ObjectPlacementSave::WallMounted {
                offset_along_segment,
                height_on_wall,
                ..
            } => {
                if !offset_along_segment.is_finite() || !height_on_wall.is_finite() {
                    issues.push(LoadIssue::ObjectPlacementInvalid { object_id: obj.id });
                    object_plans.push(ObjectLoadPlan::Skip {
                        id: obj.id,
                        reason: LoadIssue::ObjectPlacementInvalid { object_id: obj.id },
                    });
                    continue;
                }
            }
        }

        object_plans.push(ObjectLoadPlan::Spawn(obj.clone()));
    }

    let normalized_next_object_id = save.next_object_id.max(max_loaded_id + 1);
    if normalized_next_object_id > save.next_object_id {
        issues.push(LoadIssue::AllocatorNextIdTooSmall {
            save_next: save.next_object_id,
            normalized_next: normalized_next_object_id,
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

pub fn reset_tool_session(
    session: &mut ToolSessionState,
    return_state: &mut ToolReturnState,
    next_mode: &mut NextState<ToolMode>,
    selection: &mut crate::tools::SelectionState,
    build_selection: &mut BuildSelectionState,
) {
    session.active = None;
    return_state.previous = None;
    next_mode.set(ToolMode::Cursor);
    selection.primary = None;
    build_selection.selected_prototype_id = None;
}

pub fn reset_tool_runtime_flags(
    cycle: &mut PrimaryPointerCycle,
    gate: &mut ToolInputGate,
    drag: &mut PointerDragState,
    command_queue: &mut crate::store::commands::DomainCommandQueue,
) {
    cycle.owner = crate::tools::PointerPressOwner::None;
    gate.world_blocked = false;
    drag.is_camera_dragging = false;
    command_queue.commands.clear();
}

pub fn reset_ui_runtime(
    ribbon_state: &mut RibbonState,
    ui_runtime: &mut UiRuntime,
    active_panel: &mut ActiveInterfacePanel,
    window_stack: &mut UiWindowStack,
    modal_stack: &mut ModalStack,
    targets: &mut PointerTargets,
) {
    ribbon_state.is_open = false;
    ribbon_state.active_tab = BuildRibbonTab::Fixtures;
    ui_runtime.pointer_over_ui = false;
    active_panel.id = None;
    window_stack.windows.clear();
    modal_stack.stack.clear();
    targets.world_object = None;
    targets.world_widget = None;
    targets.wall_surface = None;
    targets.exterior = None;
    targets.debug = None;
}

pub fn clear_runtime_owned(commands: &mut Commands, runtime_owned: Vec<Entity>) {
    for entity in runtime_owned {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_load_plan(
    commands: &mut Commands,
    asset_server: &AssetServer,
    store: &mut StoreArea,
    allocator: &mut StableObjectIdAllocator,
    catalog: &ObjectCatalog,
    existing_objects: &Query<Entity, With<StoreObject>>,
    world_bounds: &WorldBounds,
    dirty_openings: &mut DirtyWallOpeningSegments,
    plan: LoadPlan,
) -> LoadReport {
    // 1. Clear existing objects
    for entity in existing_objects {
        commands.entity(entity).despawn();
    }

    // 2. Rebuild StoreArea
    store.owned_chunks.clear();
    for chunk in &plan.valid_chunks {
        store
            .owned_chunks
            .insert(chunk.coord, StoreChunkData { kind: chunk.kind });
    }

    // 3. Spawn objects
    let mut loaded_count = 0;
    let wall_surfaces = crate::store::boundary::collect_boundary_segments(store, world_bounds);

    for object_plan in &plan.object_plans {
        if let ObjectLoadPlan::Spawn(obj) = object_plan {
            let mut placement = placement_from_save(&obj.placement);
            let mut derived_door = None;

            if let Some(proto) = catalog.prototypes.get(&obj.prototype_id)
                && let ObjectPlacement::WallMounted { attachment } = placement
            {
                placement = ObjectPlacement::WallMounted {
                    attachment: crate::objects::prototypes::normalize_wall_attachment_for_prototype(
                        proto, attachment,
                    ),
                };
            }

            if let Some(proto) = catalog.prototypes.get(&obj.prototype_id)
                && let Some(door_spec) = crate::objects::prototypes::doorway_spec(proto)
                && let Some(spec) = crate::objects::prototypes::wall_mounted_spec(proto)
                && let ObjectPlacement::WallMounted { attachment } = placement
                && let Some(surface) = wall_surfaces
                    .iter()
                    .find(|s| s.key == attachment.segment_key)
                && let Ok(derived) = crate::store::boundary::derive_door_placement(
                    spec.width,
                    spec.height,
                    door_spec.access_width,
                    door_spec.access_depth,
                    attachment,
                    &crate::store::boundary::WallSurface {
                        key: surface.key,
                        start: surface.start,
                        end: surface.end,
                        thickness: surface.thickness,
                        height: surface.height,
                        length: surface.length,
                        normal: surface.normal,
                    },
                    crate::objects::prototypes::wall_occupancy_kind_for_prototype(proto),
                )
            {
                derived_door = Some(derived);
            }

            if let Ok(_entity) = spawn_store_object_from_prototype(
                commands,
                asset_server,
                catalog,
                SpawnStoreObjectParams {
                    stable_id: obj.id,
                    prototype_id: obj.prototype_id.clone(),
                    placement,
                    derived_door,
                },
            ) {
                loaded_count += 1;
                if let ObjectPlacement::WallMounted { attachment } = placement {
                    if catalog
                        .prototypes
                        .get(&obj.prototype_id)
                        .and_then(wall_opening_spec)
                        .is_some()
                    {
                        dirty_openings.dirty.insert(attachment.segment_key);
                    }
                }
            }
        }
    }

    // 4. Update allocator
    allocator.next = plan.normalized_next_object_id;

    let mut report = plan.report;
    report.loaded_objects = loaded_count;
    report
}

fn placement_from_save(save: &ObjectPlacementSave) -> ObjectPlacement {
    match save {
        ObjectPlacementSave::Floor {
            world_pos,
            rotation_index,
        } => ObjectPlacement::Floor {
            world_pos: Vec2::new(world_pos.x, world_pos.y),
            rotation_index: *rotation_index,
        },
        ObjectPlacementSave::WallMounted {
            segment_key,
            offset_along_segment,
            height_on_wall,
        } => ObjectPlacement::WallMounted {
            attachment: WallAttachmentPoint {
                segment_key: WallSegmentKey {
                    chunk: segment_key.chunk,
                    side: segment_key.side,
                },
                offset_along_segment: *offset_along_segment,
                height_on_wall: *height_on_wall,
            },
        },
    }
}

#[cfg(test)]
mod tests;
