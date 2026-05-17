pub mod build;
pub mod context;
pub mod cursor;
pub mod delete;
pub mod expansion;
pub mod gate;
pub mod mode;
pub mod move_tool;
pub mod preview;
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
pub use session::*;

use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::objects::components::*;
use crate::objects::prototypes::{BuildPrototypeId, spawn_object_from_prototype};
use crate::objects::rotation::Rotatable;
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
        app.add_plugins(ToolPreviewPlugin)
            .init_resource::<ToolContext>()
            .init_resource::<ToolInputGate>()
            .init_resource::<PrimaryPointerCycle>()
            .init_resource::<ToolRegistry>()
            .init_resource::<ToolSessionState>()
            .init_resource::<ToolReturnState>()
            .add_message::<ObjectActionRequested>()
            .add_message::<MoveObjectCommitted>()
            .add_message::<DeleteObjectRequested>()
            .add_message::<BuildObjectRequested>()
            .add_message::<StartMoveObjectRequested>()
            .add_message::<ToolChangedRequested>()
            .add_message::<ActivateToolRequested>()
            .add_message::<ReturnToPreviousToolRequested>()
            .configure_sets(
                Update,
                (
                    ToolSet::InputGate,
                    ToolSet::ToolUpdate,
                    ToolSet::Validation,
                    ToolSet::Commit,
                )
                    .chain(),
            )
            .add_systems(Update, tool_hotkeys_system)
            .add_systems(
                Update,
                (
                    update_tool_input_gate.after(crate::input::camera_drag_system),
                    handle_activate_tool_requested,
                    handle_return_to_previous_tool_requested,
                )
                    .chain()
                    .in_set(ToolSet::InputGate),
            )
            .add_systems(
                Update,
                unified_tool_validation_system.in_set(ToolSet::Validation),
            )
            .add_systems(
                Update,
                (
                    apply_committed_events,
                    crate::store::apply_purchase_store_chunk_requested,
                    log_tool_changed_requests,
                    print_positions_system,
                )
                    .chain()
                    .in_set(ToolSet::Commit),
            );
    }
}

fn unified_tool_validation_system(
    world_bounds: Res<WorldBounds>,
    store_area: Res<StoreArea>,
    mut session: ResMut<ToolSessionState>,
    footprints: Query<(Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>)>,
    mut previews: Query<(&mut PlacementPreview, &WorldPos, &Footprint)>,
) {
    let Some(active) = session.active.as_mut() else {
        return;
    };

    match active {
        ActiveToolSession::Build(build) => {
            if let Ok((mut preview, pos, footprint)) = previews.get_mut(build.preview_entity) {
                let result = crate::placement::validate_placement(
                    &world_bounds,
                    &store_area,
                    &footprints,
                    footprint,
                    pos.0,
                    crate::placement::PlacementValidationOptions::default(),
                );
                preview.validation = Some(result);
            }
        }
        ActiveToolSession::Move(move_session) => {
            if let Ok((mut preview, pos, footprint)) = previews.get_mut(move_session.preview_entity) {
                let result = crate::placement::validate_placement(
                    &world_bounds,
                    &store_area,
                    &footprints,
                    footprint,
                    pos.0,
                    crate::placement::PlacementValidationOptions {
                        ignore_entity: Some(move_session.source_entity),
                    },
                );
                preview.validation = Some(result);
            }
        }
        _ => {}
    }
}

fn handle_activate_tool_requested(
    mut commands: Commands,
    mut events: MessageReader<ActivateToolRequested>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    current_mode: Res<State<ToolMode>>,
    mut session: ResMut<ToolSessionState>,
    mut return_state: ResMut<ToolReturnState>,
) {
    for event in events.read() {
        if event.mode == *current_mode.get() {
            continue;
        }

        info!(
            "Activating tool {:?} ({:?})",
            event.mode, event.kind
        );

        if event.kind == ToolActivationKind::Replace {
            return_state.previous = None;
        } else if event.kind == ToolActivationKind::Temporary {
            return_state.previous = Some(*current_mode.get());
        }

        cleanup_current_session(&mut commands, &mut session, ToolSessionEndReason::Replaced);

        next_mode.set(event.mode);
    }
}

fn handle_return_to_previous_tool_requested(
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

    info!("Cleaning up tool session {:?} (Reason: {:?})", active, reason);

    match active {
        ActiveToolSession::Build(s) => {
            commands.entity(s.preview_entity).despawn();
        }
        ActiveToolSession::Move(s) => {
            commands.entity(s.preview_entity).despawn();
            commands.entity(s.source_entity).remove::<PreviewSource>();
        }
        ActiveToolSession::Expansion(_) => {}
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ObjectActionRequested {
    pub entity: Entity,
    pub action: ObjectAction,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectAction {
    Select,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct MoveObjectCommitted {
    pub entity: Entity,
    pub old_pos: Vec2,
    pub new_pos: Vec2,
    pub rotation: usize,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DeleteObjectRequested {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BuildObjectRequested {
    pub prototype: BuildPrototypeId,
    pub pos: Vec2,
    pub rotation: usize,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct StartMoveObjectRequested {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ToolChangedRequested {
    pub mode: ToolMode,
}

pub fn apply_committed_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_bounds: Res<WorldBounds>,
    store_area: Res<StoreArea>,
    mut object_actions: MessageReader<ObjectActionRequested>,
    mut moves: MessageReader<MoveObjectCommitted>,
    mut deletes: MessageReader<DeleteObjectRequested>,
    mut builds: MessageReader<BuildObjectRequested>,
    mut selected: Query<Entity, With<Selected>>,
    mut tool: ResMut<ToolContext>,
    mut set: ParamSet<(
        Query<(Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>)>,
        Query<(&mut Rotatable, &mut Sprite, &mut Footprint, &mut FootAnchor, &mut VisualOffset), Without<ToolPreview>>,
    )>,
) {
    for action in object_actions.read() {
        info!(
            "ObjectActionRequested entity={:?} action={:?}",
            action.entity, action.action
        );
        if matches!(action.action, ObjectAction::Select) {
            for entity in &mut selected {
                commands.entity(entity).remove::<Selected>();
            }
            commands.entity(action.entity).insert(Selected);
        }
    }

    for movement in moves.read() {
        // 1. Revalidate Move using p0
        let validation = {
            let footprints = set.p0();
            let Ok((_, _, footprint, _)) = footprints.get(movement.entity) else {
                continue;
            };

            crate::placement::validate_placement(
                &world_bounds,
                &store_area,
                &footprints,
                footprint,
                movement.new_pos,
                crate::placement::PlacementValidationOptions {
                    ignore_entity: Some(movement.entity),
                },
            )
        };

        if validation.is_ok() {
            if let Ok(mut e) = commands.get_entity(movement.entity) {
                e.insert(WorldPos(movement.new_pos));
            }

            // 2. Apply final rotation using p1
            if let Ok((mut rotatable, mut sprite, mut fp, mut anchor, mut offset)) = set.p1().get_mut(movement.entity) {
                if movement.rotation < rotatable.variants.len() {
                    rotatable.current = movement.rotation;
                    let variant = &rotatable.variants[rotatable.current];
                    sprite.image = variant.sprite.clone();
                    *fp = variant.footprint.clone();
                    anchor.0 = variant.foot_anchor;
                    offset.0 = variant.visual_offset;
                }
            }

            info!(
                "MoveObjectCommitted entity={:?} old=({:.1},{:.1}) new=({:.1},{:.1}) rotation={}",
                movement.entity,
                movement.old_pos.x,
                movement.old_pos.y,
                movement.new_pos.x,
                movement.new_pos.y,
                movement.rotation
            );
        } else {
            warn!(
                "MoveObjectCommitted REJECTED for entity={:?}: {:?}",
                movement.entity, validation.err()
            );
        }
    }

    for delete in deletes.read() {
        if let Some(active) = tool.active {
            let active_entity = match active {
                ActiveToolAction::Moving { entity, .. }
                | ActiveToolAction::PendingDelete { entity } => Some(entity),
                ActiveToolAction::Building { ghost, .. } => Some(ghost),
            };
            if active_entity == Some(delete.entity) {
                tool.active = None;
            }
        }
        commands.entity(delete.entity).despawn();
        info!("DeleteObjectRequested entity={:?}", delete.entity);
    }

    for build in builds.read() {
        // Revalidate Build using p0
        let spec = crate::objects::prototypes::prototype_spec(build.prototype);
        let footprint = Footprint::rectangle(spec.footprint_half_extents);

        let validation = crate::placement::validate_placement(
            &world_bounds,
            &store_area,
            &set.p0(),
            &footprint,
            build.pos,
            crate::placement::PlacementValidationOptions::default(),
        );

        if validation.is_ok() {
            spawn_object_from_prototype(&mut commands, &asset_server, build.prototype, build.pos, build.rotation);
            info!(
                "BuildObjectRequested prototype={:?} pos=({:.1},{:.1}) rotation={}",
                build.prototype, build.pos.x, build.pos.y, build.rotation
            );
        } else {
            warn!(
                "BuildObjectRequested REJECTED for prototype={:?}: {:?}",
                build.prototype, validation.err()
            );
        }
    }
}

pub fn print_positions_system(
    actions: Res<InputActionState>,
    query: Query<(&PlaceableAssetId, &WorldPos, &SortLayer, &FootAnchor), Without<BuildGhost>>,
) {
    if !actions.just_pressed(InputAction::PrintDebugPositions) {
        return;
    }

    info!("--- placeable positions ---");
    for (asset_id, world_pos, sort_layer, foot_anchor) in &query {
        info!(
            "asset_id={} world_x={:.2} world_y={:.2} sort_layer={:?} foot_anchor=({:.2},{:.2})",
            asset_id.0, world_pos.0.x, world_pos.0.y, sort_layer, foot_anchor.0.x, foot_anchor.0.y
        );
    }
}
