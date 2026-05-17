use bevy::prelude::*;

use crate::input::{InputAction, PointerContext};
use crate::objects::components::{BuildGhost, GhostOf, WorldPos};
use crate::objects::prototypes::{BuildPrototypeId, BuildPrototypes, spawn_ghost_from_prototype};
use crate::tools::{
    ActiveToolAction, BuildObjectRequested, ToolContext, ToolDescriptor, ToolInputGate, ToolMode,
    ToolRegistry, ToolSet,
};

pub struct BuildToolPlugin;

impl Plugin for BuildToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Build,
                action: InputAction::ToolBuild,
                label: "Build",
            });

        app.add_message::<SelectBuildObjectRequested>()
            .add_systems(OnEnter(ToolMode::Build), spawn_build_ghost)
            .add_systems(OnExit(ToolMode::Build), despawn_build_ghost)
            .add_systems(
                Update,
                (
                    apply_select_build_object_requests,
                    build_tool_system.run_if(in_state(ToolMode::Build)),
                )
                    .chain()
                    .in_set(ToolSet::ToolUpdate),
            );
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SelectBuildObjectRequested {
    pub prototype: BuildPrototypeId,
}

fn apply_select_build_object_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pointer: Res<PointerContext>,
    mode: Res<State<ToolMode>>,
    mut requests: MessageReader<SelectBuildObjectRequested>,
    mut prototypes: ResMut<BuildPrototypes>,
    mut tool: ResMut<ToolContext>,
    ghosts: Query<Entity, With<BuildGhost>>,
    mut next_mode: ResMut<NextState<ToolMode>>,
) {
    for request in requests.read() {
        prototypes.active = request.prototype;
        next_mode.set(ToolMode::Build);

        if *mode.get() != ToolMode::Build {
            continue;
        }

        for ghost in &ghosts {
            commands.entity(ghost).despawn();
        }
        let ghost = spawn_ghost_from_prototype(
            &mut commands,
            &asset_server,
            request.prototype,
            pointer.world_pos,
        );
        tool.active = Some(ActiveToolAction::Building {
            prototype: request.prototype,
            ghost,
            current_world_pos: pointer.world_pos,
            valid: false,
        });
    }
}

pub fn spawn_build_ghost(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    prototypes: Res<BuildPrototypes>,
    pointer: Res<PointerContext>,
    mut tool: ResMut<ToolContext>,
) {
    let ghost = spawn_ghost_from_prototype(
        &mut commands,
        &asset_server,
        prototypes.active,
        pointer.world_pos,
    );
    tool.active = Some(ActiveToolAction::Building {
        prototype: prototypes.active,
        ghost,
        current_world_pos: pointer.world_pos,
        valid: false,
    });
}

pub fn build_tool_system(
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    mut tool: ResMut<ToolContext>,
    mut ghost_positions: Query<&mut WorldPos, With<BuildGhost>>,
    mut builds: MessageWriter<BuildObjectRequested>,
) {
    tool.sync_from_pointer(&pointer);

    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        tool.active = None;
        next_mode.set(ToolMode::Cursor);
        return;
    }

    let pointer_world = tool.pointer_world;
    if let Some(ActiveToolAction::Building {
        prototype,
        ghost,
        current_world_pos,
        valid,
    }) = tool.active.as_mut()
    {
        *current_world_pos = pointer_world;
        if let Ok(mut world_pos) = ghost_positions.get_mut(*ghost) {
            world_pos.0 = *current_world_pos;
        }

        if gate.primary_click_released && *valid {
            builds.write(BuildObjectRequested {
                prototype: *prototype,
                pos: *current_world_pos,
            });
        }
    }
}

pub fn despawn_build_ghost(
    mut commands: Commands,
    mut tool: ResMut<ToolContext>,
    ghosts: Query<(Entity, Option<&GhostOf>), With<BuildGhost>>,
) {
    for (entity, ghost_of) in &ghosts {
        if let Some(ghost_of) = ghost_of {
            info!(
                "Despawning build ghost for prototype={:?}",
                ghost_of.prototype
            );
        }
        commands.entity(entity).despawn();
    }

    if matches!(tool.active, Some(ActiveToolAction::Building { .. })) {
        tool.active = None;
    }
}
