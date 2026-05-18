pub mod build;
pub mod context;
pub mod cursor;
pub mod delete;
pub mod expansion;
pub mod gate;
pub mod mode;
pub mod move_tool;
pub mod preview;
pub mod selection;
pub mod session;

pub use build::*;
pub use context::*;
pub use cursor::*;
pub use delete::*;
pub use expansion::*;
pub use gate::*;
pub use mode::*;
pub use move_tool::*;
pub use preview::*;
pub use selection::*;
pub use session::*;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::objects::components::*;
use crate::objects::prototypes::BuildObjectId;
use crate::objects::rotation::RotateObjectRequested;
use crate::store::commands::{
    BuildObjectCommand, DeleteObjectCommand, DomainCommand, DomainCommandQueue, MoveObjectCommand,
};
use crate::store::{StoreArea, WorldBounds};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolSet {
    InputGate,
    ToolUpdate,
    Validation,
    Commit,
}

pub struct ToolCorePlugin;

impl Plugin for ToolCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ToolPreviewPlugin, SelectionPlugin))
            .init_resource::<ToolContext>()
            .init_resource::<ToolInputGate>()
            .init_resource::<PrimaryPointerCycle>()
            .init_resource::<ToolRegistry>()
            .init_resource::<ToolSessionState>()
            .init_resource::<ToolReturnState>()
            .insert_resource(StableObjectIdAllocator { next: 1 })
            .add_message::<ObjectActionRequested>()
            .add_message::<MoveObjectCommitted>()
            .add_message::<DeleteObjectRequested>()
            .add_message::<BuildObjectRequested>()
            .add_message::<StartMoveObjectRequested>()
            .add_message::<ToolChangedRequested>()
            .add_message::<ActivateToolRequested>()
            .add_message::<ReturnToPreviousToolRequested>();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn unified_tool_validation_system(
    world_bounds: Res<WorldBounds>,
    store_area: Res<StoreArea>,
    mut session: ResMut<ToolSessionState>,
    footprints: Query<
        (
            Entity,
            &crate::objects::components::WorldPos,
            &crate::objects::components::Footprint,
            Option<&crate::objects::components::BlocksPlacement>,
        ),
        Without<crate::objects::components::WallMounted>,
    >,
    access_zones: Query<(
        Entity,
        &crate::objects::components::InteriorAccessZone,
        Option<&crate::objects::components::ObjectStableId>,
    )>,
    stable_ids: Query<&crate::objects::components::ObjectStableId>,
    mut previews: Query<(
        &mut PlacementPreview,
        &WorldPos,
        Option<&Footprint>,
        Option<&crate::tools::WallMountedPreview>,
    )>,
    wallprints: Query<(
        &crate::objects::components::Wallprint,
        &crate::objects::components::ObjectStableId,
    )>,
) {
    let Some(active) = session.active.as_mut() else {
        return;
    };

    match active {
        ActiveToolSession::Build(build) => match build {
            BuildToolSession::Floor(floor) => {
                if let Ok((mut preview, pos, Some(footprint), _)) =
                    previews.get_mut(floor.preview_entity)
                {
                    let result = crate::placement::validate_placement(
                        &world_bounds,
                        &store_area,
                        &footprints,
                        &access_zones,
                        footprint,
                        pos.0,
                        crate::placement::PlacementValidationOptions::default(),
                    );
                    preview.validation = Some(result);
                }
            }
            BuildToolSession::WallMounted(wall) => {
                if let Ok((mut preview, _, _, Some(_))) = previews.get_mut(wall.preview_entity) {
                    preview.validation = Some(match wall.current_attachment {
                        Some(_) => Ok(()),
                        None => Err(crate::store::PlacementInvalidReason::WallSurfaceMissing),
                    });
                }
            }
        },
        ActiveToolSession::Move(move_session) => match move_session {
            crate::tools::MoveToolSession::Floor(s) => {
                if let Ok((mut preview, pos, Some(footprint), _)) =
                    previews.get_mut(s.preview_entity)
                {
                    let stable_id = stable_ids.get(s.source_entity).ok().map(|id| id.0);
                    let result = crate::placement::validate_placement(
                        &world_bounds,
                        &store_area,
                        &footprints,
                        &access_zones,
                        footprint,
                        pos.0,
                        crate::placement::PlacementValidationOptions {
                            ignore_entity: Some(s.source_entity),
                            ignore_stable_id: stable_id,
                        },
                    );
                    preview.validation = Some(result);
                }
            }
            crate::tools::MoveToolSession::WallMounted(s) => {
                if let Ok((mut preview, _, _, _)) = previews.get_mut(s.preview_entity) {
                    preview.validation = Some(match s.current_attachment {
                        Some(_) => Ok(()),
                        None => Err(crate::store::PlacementInvalidReason::WallSurfaceMissing),
                    });
                }
            }
            crate::tools::MoveToolSession::Door(s) => {
                if let Ok((mut preview, _, _, _)) = previews.get_mut(s.preview_entity) {
                    preview.validation = Some(match s.current_attachment {
                        Some(_) => match &s.current_derived {
                            Some(derived) => {
                                let stable_id =
                                    wallprints.get(s.source_entity).ok().map(|(_, id)| id.0);
                                crate::placement::validate_derived_door_placement(
                                    derived,
                                    &store_area,
                                    &footprints,
                                    &wallprints,
                                    crate::placement::PlacementValidationOptions {
                                        ignore_entity: Some(s.source_entity),
                                        ignore_stable_id: stable_id,
                                    },
                                )
                            }
                            None => Err(crate::store::PlacementInvalidReason::WallSurfaceMissing),
                        },
                        None => Err(crate::store::PlacementInvalidReason::WallSurfaceMissing),
                    });
                }
            }
        },
        _ => {}
    }
}

#[derive(Message, Debug, Clone)]
pub struct ActivateToolRequested {
    pub mode: ToolMode,
    pub kind: ToolActivationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolActivationKind {
    Replace,
    Temporary,
}

pub fn handle_activate_tool_requested(
    mut commands: Commands,
    mut events: MessageReader<ActivateToolRequested>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    current_mode: Res<State<ToolMode>>,
    mut session: ResMut<ToolSessionState>,
    mut return_state: ResMut<ToolReturnState>,
) {
    for event in events.read() {
        info!("Activating tool {:?} ({:?})", event.mode, event.kind);

        if event.kind == ToolActivationKind::Replace {
            return_state.previous = None;
        } else if event.kind == ToolActivationKind::Temporary {
            return_state.previous = Some(*current_mode.get());
        }

        cleanup_current_session(&mut commands, &mut session, ToolSessionEndReason::Replaced);

        next_mode.set(event.mode);
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ReturnToPreviousToolRequested;

pub fn handle_return_to_previous_tool_requested(
    mut commands: Commands,
    mut events: MessageReader<ReturnToPreviousToolRequested>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    mut session: ResMut<ToolSessionState>,
    mut return_state: ResMut<ToolReturnState>,
) {
    for _ in events.read() {
        let previous = return_state.previous.take().unwrap_or(ToolMode::Cursor);
        info!("Returning to previous tool {:?}", previous);

        cleanup_current_session(&mut commands, &mut session, ToolSessionEndReason::Returned);

        next_mode.set(previous);
    }
}

pub fn cleanup_current_session(
    commands: &mut Commands,
    session: &mut ToolSessionState,
    reason: ToolSessionEndReason,
) {
    let Some(active) = session.active.take() else {
        return;
    };

    info!(
        "Cleaning up tool session {:?} (Reason: {:?})",
        active, reason
    );

    match active {
        ActiveToolSession::Build(s) => {
            commands.entity(s.preview_entity()).try_despawn();
            if let BuildToolSession::WallMounted(w) = s
                && let Some(az) = w.access_zone_preview_entity
            {
                commands.entity(az).try_despawn();
            }
        }
        ActiveToolSession::Move(s) => {
            let (preview_entity, source_entity, access_zone_entity) = match s {
                crate::tools::MoveToolSession::Floor(f) => {
                    (f.preview_entity, f.source_entity, None)
                }
                crate::tools::MoveToolSession::WallMounted(w) => {
                    (w.preview_entity, w.source_entity, None)
                }
                crate::tools::MoveToolSession::Door(d) => (
                    d.preview_entity,
                    d.source_entity,
                    d.access_zone_preview_entity,
                ),
            };
            commands.entity(preview_entity).try_despawn();
            if let Some(az) = access_zone_entity {
                commands.entity(az).try_despawn();
            }
            commands.entity(source_entity).try_remove::<PreviewSource>();
        }
        ActiveToolSession::Expansion(_) => {}
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ObjectActionRequested {
    pub entity: Entity,
    pub action: ObjectActionKind,
    #[allow(dead_code)]
    pub origin: ObjectActionOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectActionKind {
    Inspect,
    Deselect,
    Move,
    Rotate,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectActionOrigin {
    CursorClick,
    InspectorButton,
    WorldWidget,
    #[allow(dead_code)]
    Hotkey,
}

pub fn handle_object_action_requests(mut params: ObjectActionRequestParams) {
    for request in params.requests.read() {
        info!("ObjectActionRequested: {:?}", request);
        match request.action {
            ObjectActionKind::Inspect => {
                params.selection.primary = Some(request.entity);
            }
            ObjectActionKind::Deselect => {
                if params.selection.primary == Some(request.entity) {
                    params.selection.primary = None;
                }
            }
            ObjectActionKind::Move => {
                // Ensure tool is active
                params.tool_activation.write(ActivateToolRequested {
                    mode: ToolMode::Move,
                    kind: ToolActivationKind::Replace,
                });
                params.move_requests.write(StartMoveObjectRequested {
                    entity: request.entity,
                });
            }
            ObjectActionKind::Rotate => {
                params.rotate_requests.write(RotateObjectRequested {
                    entity: request.entity,
                    steps: 1,
                });
                if request.origin == ObjectActionOrigin::WorldWidget {
                    params.move_requests.write(StartMoveObjectRequested {
                        entity: request.entity,
                    });
                }
            }
            ObjectActionKind::Delete => {
                params.modal_requests.write(crate::ui::ModalRequest::Open(
                    crate::ui::ModalKind::ConfirmDelete {
                        entity: request.entity,
                    },
                ));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct ObjectActionRequestParams<'w, 's> {
    requests: MessageReader<'w, 's, ObjectActionRequested>,
    selection: ResMut<'w, SelectionState>,
    move_requests: MessageWriter<'w, StartMoveObjectRequested>,
    rotate_requests: MessageWriter<'w, RotateObjectRequested>,
    modal_requests: MessageWriter<'w, crate::ui::ModalRequest>,
    tool_activation: MessageWriter<'w, ActivateToolRequested>,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct MoveObjectCommitted {
    pub entity: Entity,
    pub new_placement: crate::objects::components::ObjectPlacement,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DeleteObjectRequested {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct BuildObjectRequested {
    pub prototype: BuildObjectId,
    pub placement: crate::objects::components::ObjectPlacement,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct StartMoveObjectRequested {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ToolChangedRequested {
    pub mode: ToolMode,
}

pub fn convert_committed_requests_to_commands(
    mut queue: ResMut<DomainCommandQueue>,
    mut allocator: ResMut<StableObjectIdAllocator>,
    mut moves: MessageReader<MoveObjectCommitted>,
    mut deletes: MessageReader<DeleteObjectRequested>,
    mut builds: MessageReader<BuildObjectRequested>,
    stable_ids: Query<&ObjectStableId>,
    _world_positions: Query<&WorldPos>,
) {
    for movement in moves.read() {
        if let Ok(stable_id) = stable_ids.get(movement.entity) {
            queue
                .commands
                .push_back(DomainCommand::MoveObject(MoveObjectCommand {
                    object_id: stable_id.0,
                    new_placement: movement.new_placement,
                }));
        }
    }

    for delete in deletes.read() {
        if let Ok(stable_id) = stable_ids.get(delete.entity) {
            queue
                .commands
                .push_back(DomainCommand::DeleteObject(DeleteObjectCommand {
                    object_id: stable_id.0,
                }));
        }
    }

    for build in builds.read() {
        let stable_id = allocator.allocate();
        queue
            .commands
            .push_back(DomainCommand::BuildObject(BuildObjectCommand {
                object_id: stable_id,
                prototype_id: build.prototype.clone(),
                placement: build.placement,
            }));
    }
}

pub fn handle_domain_event_selection_cleanup(
    mut events: MessageReader<crate::store::events::DomainEvent>,
    mut selection: ResMut<SelectionState>,
    stable_ids: Query<(Entity, &ObjectStableId)>,
) {
    for event in events.read() {
        if let crate::store::events::DomainEvent::ObjectDeleted { id } = event
            && let Some(selected_entity) = selection.primary
            && let Ok((_, stable_id)) = stable_ids.get(selected_entity)
            && &stable_id.0 == id
        {
            info!("Clearing selection for deleted object {:?}", id);
            selection.primary = None;
        }
    }
}

pub fn print_positions_system(
    actions: Res<InputActionState>,
    query: Query<(&ObjectPrototypeId, &WorldPos, &SortLayer, &FootAnchor), Without<BuildGhost>>,
) {
    if !actions.just_pressed(InputAction::PrintDebugPositions) {
        return;
    }

    info!("--- placeable positions ---");
    for (prototype_id, world_pos, sort_layer, foot_anchor) in &query {
        info!(
            "prototype_id={} world_x={:.2} world_y={:.2} sort_layer={:?} foot_anchor=({:.2},{:.2})",
            prototype_id.0.0,
            world_pos.0.x,
            world_pos.0.y,
            sort_layer,
            foot_anchor.0.x,
            foot_anchor.0.y
        );
    }
}
