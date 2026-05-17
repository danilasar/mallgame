pub mod build;
pub mod context;
pub mod cursor;
pub mod delete;
pub mod gate;
pub mod mode;
pub mod move_tool;

pub use build::*;
pub use context::*;
pub use cursor::*;
pub use delete::*;
pub use gate::*;
pub use mode::*;
pub use move_tool::*;

use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::objects::components::*;
use crate::objects::prototypes::{BuildPrototypeId, spawn_object_from_prototype};

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
        app.init_resource::<ToolContext>()
            .init_resource::<ToolInputGate>()
            .init_resource::<ToolRegistry>()
            .add_message::<ObjectActionRequested>()
            .add_message::<MoveObjectCommitted>()
            .add_message::<DeleteObjectRequested>()
            .add_message::<BuildObjectRequested>()
            .add_message::<StartMoveObjectRequested>()
            .add_message::<ToolChangedRequested>()
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
                update_tool_input_gate
                    .after(crate::input::camera_drag_system)
                    .in_set(ToolSet::InputGate),
            )
            .add_systems(
                Update,
                crate::placement::validate_active_placement.in_set(ToolSet::Validation),
            )
            .add_systems(
                Update,
                (
                    apply_committed_events,
                    log_tool_changed_requests,
                    print_positions_system,
                )
                    .chain()
                    .in_set(ToolSet::Commit),
            );
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
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DeleteObjectRequested {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BuildObjectRequested {
    pub prototype: BuildPrototypeId,
    pub pos: Vec2,
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
    mut object_actions: MessageReader<ObjectActionRequested>,
    mut moves: MessageReader<MoveObjectCommitted>,
    mut deletes: MessageReader<DeleteObjectRequested>,
    mut builds: MessageReader<BuildObjectRequested>,
    mut selected: Query<Entity, With<Selected>>,
    mut world_pos: Query<&mut WorldPos>,
    mut tool: ResMut<ToolContext>,
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
        if let Ok(mut pos) = world_pos.get_mut(movement.entity) {
            pos.0 = movement.new_pos;
        }
        info!(
            "MoveObjectCommitted entity={:?} old=({:.1},{:.1}) new=({:.1},{:.1})",
            movement.entity,
            movement.old_pos.x,
            movement.old_pos.y,
            movement.new_pos.x,
            movement.new_pos.y
        );
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
        spawn_object_from_prototype(&mut commands, &asset_server, build.prototype, build.pos);
        info!(
            "BuildObjectRequested prototype={:?} pos=({:.1},{:.1})",
            build.prototype, build.pos.x, build.pos.y
        );
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
