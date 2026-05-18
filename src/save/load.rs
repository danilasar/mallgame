use crate::input::{PointerDragState, PointerTargets};
use crate::objects::components::{
    ObjectPlacement, StableObjectIdAllocator, StoreObject, WallAttachmentPoint,
};
use crate::objects::prototypes::{
    BuildRibbonTab, BuildSelectionState, ObjectCatalog, SpawnStoreObjectParams,
    spawn_store_object_from_prototype,
};
use crate::save::types::*;
use crate::save::validation::*;
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

pub fn apply_load_plan(
    commands: &mut Commands,
    asset_server: &AssetServer,
    store: &mut StoreArea,
    allocator: &mut StableObjectIdAllocator,
    catalog: &ObjectCatalog,
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
        store
            .owned_chunks
            .insert(chunk.coord, StoreChunkData { kind: chunk.kind });
    }

    // 3. Spawn objects
    let mut loaded_count = 0;
    for object_plan in &plan.object_plans {
        if let ObjectLoadPlan::Spawn(obj) = object_plan
            && let Ok(_entity) = spawn_store_object_from_prototype(
                commands,
                asset_server,
                catalog,
                SpawnStoreObjectParams {
                    stable_id: obj.id,
                    prototype_id: obj.prototype_id.clone(),
                    placement: placement_from_save(&obj.placement),
                },
            )
        {
            loaded_count += 1;
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
mod tests {
    use super::*;
    use crate::objects::components::*;
    use crate::objects::prototypes::BuildObjectId;

    #[test]
    fn test_save_load_restores_capabilities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        crate::store::commands::register_test_messages(&mut app);

        // Setup catalog
        let commands = app.world_mut().commands();
        crate::objects::prototypes::setup_object_catalog(commands);
        app.update();

        let _catalog = app.world().resource::<ObjectCatalog>().clone();

        let save = SaveGame {
            version: CURRENT_SAVE_VERSION,
            next_object_id: 2000,
            store: StoreSave {
                owned_chunks: vec![],
            },
            objects: vec![ObjectSave {
                id: StableObjectId(1001),
                prototype_id: BuildObjectId::new("fixture.shelf.basic"),
                placement: ObjectPlacementSave::Floor {
                    world_pos: WorldPosSave { x: 0.0, y: 0.0 },
                    rotation_index: None,
                },
            }],
        };

        let plan =
            build_load_plan(save, &SaveLoadLimits::default(), &WorldBounds::default()).unwrap();

        // Mock resources for apply_load_plan
        app.insert_resource(StoreArea::new(Vec2::ZERO));
        app.insert_resource(StableObjectIdAllocator { next: 1 });

        // Simpler way in App tests: run it as a system
        app.world_mut().insert_resource(plan);
        app.add_systems(
            Update,
            |mut commands: Commands,
             asset_server: Res<AssetServer>,
             mut store: ResMut<StoreArea>,
             mut allocator: ResMut<StableObjectIdAllocator>,
             catalog: Res<ObjectCatalog>,
             query: Query<Entity, With<StoreObject>>,
             plan_res: Res<LoadPlan>| {
                apply_load_plan(
                    &mut commands,
                    &asset_server,
                    &mut store,
                    &mut allocator,
                    &catalog,
                    &query,
                    plan_res.clone(),
                );
            },
        );

        app.update();

        let world = app.world_mut();
        let mut query =
            world.query::<(&ObjectStableId, &ProductContainer, &NpcInteractionPoints)>();

        let mut found = false;
        for (sid, _, _) in query.iter(world) {
            if sid.0 == StableObjectId(1001) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "Loaded object should have sid 1001 and all capability components"
        );
    }

    #[test]
    fn test_save_load_restores_wall_mounted_object() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Image>();
        crate::store::commands::register_test_messages(&mut app);

        let commands = app.world_mut().commands();
        crate::objects::prototypes::setup_object_catalog(commands);
        app.update();

        let segment_key = crate::store::WallSegmentKey {
            chunk: crate::store::StoreChunkCoord { x: -1, y: -1 },
            side: crate::store::StoreBoundarySide::Top,
        };
        let save = SaveGame {
            version: CURRENT_SAVE_VERSION,
            next_object_id: 2000,
            store: StoreSave {
                owned_chunks: vec![],
            },
            objects: vec![ObjectSave {
                id: StableObjectId(3001),
                prototype_id: BuildObjectId::new("wall.decor.placeholder"),
                placement: ObjectPlacementSave::WallMounted {
                    segment_key: WallSegmentKeySave {
                        chunk: segment_key.chunk,
                        side: segment_key.side,
                    },
                    offset_along_segment: 64.0,
                    height_on_wall: 48.0,
                },
            }],
        };

        let plan =
            build_load_plan(save, &SaveLoadLimits::default(), &WorldBounds::default()).unwrap();

        app.insert_resource(StoreArea::new(Vec2::ZERO));
        app.insert_resource(StableObjectIdAllocator { next: 1 });
        app.world_mut().insert_resource(plan);
        app.add_systems(
            Update,
            |mut commands: Commands,
             asset_server: Res<AssetServer>,
             mut store: ResMut<StoreArea>,
             mut allocator: ResMut<StableObjectIdAllocator>,
             catalog: Res<ObjectCatalog>,
             query: Query<Entity, With<StoreObject>>,
             plan_res: Res<LoadPlan>| {
                apply_load_plan(
                    &mut commands,
                    &asset_server,
                    &mut store,
                    &mut allocator,
                    &catalog,
                    &query,
                    plan_res.clone(),
                );
            },
        );

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<(
            &ObjectStableId,
            &WallMounted,
            &crate::objects::components::WallMountedPlacement,
            &crate::objects::components::Wallprint,
            &WallMountedBounds,
            &ObjectPlacementComponent,
            Option<&Footprint>,
            &StoreObject,
        )>();
        let found = query.iter(world).any(
            |(
                sid,
                mounted,
                mounted_placement,
                wallprint,
                bounds,
                placement,
                footprint,
                _store_object,
            )| {
                sid.0 == StableObjectId(3001)
                    && mounted.attachment.segment_key == segment_key
                    && mounted_placement.attachment.segment_key == segment_key
                    && wallprint.rects.len() == 1
                    && wallprint.rects[0].segment_key == segment_key
                    && bounds.segment_key == segment_key
                    && footprint.is_none()
                    && matches!(
                        placement.placement,
                        ObjectPlacement::WallMounted { attachment }
                            if attachment.segment_key == segment_key
                    )
            },
        );

        assert!(
            found,
            "wall-mounted object should restore from save placement"
        );
    }
}
