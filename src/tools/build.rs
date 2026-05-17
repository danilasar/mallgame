use bevy::prelude::*;

use crate::input::{InputAction, PointerContext};
use crate::objects::components::WorldPos;
use crate::objects::prototypes::{BuildPrototypeId, BuildPrototypes, spawn_ghost_from_prototype};
use crate::tools::{
    ActivateToolRequested, ActiveToolSession, BuildObjectRequested, BuildToolSession,
    ToolActivationKind, ToolContext, ToolDescriptor, ToolInputGate, ToolMode, ToolRegistry,
    ToolSessionState, ToolSet, ToolPreview, ToolPreviewKind, NonInteractive, PlacementPreview,
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
            .add_systems(OnEnter(ToolMode::Build), start_build_session)
            .add_systems(OnExit(ToolMode::Build), cleanup_build_session)
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
    mut requests: MessageReader<SelectBuildObjectRequested>,
    mut prototypes: ResMut<BuildPrototypes>,
    mut activation: MessageWriter<ActivateToolRequested>,
) {
    for request in requests.read() {
        prototypes.active = request.prototype;
        activation.write(ActivateToolRequested {
            mode: ToolMode::Build,
            kind: ToolActivationKind::Replace,
        });
    }
}

fn start_build_session(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    prototypes: Res<BuildPrototypes>,
    pointer: Res<PointerContext>,
    mut session: ResMut<ToolSessionState>,
) {
    let preview_entity = spawn_ghost_from_prototype(
        &mut commands,
        &asset_server,
        prototypes.active,
        pointer.world_pos,
    );
    
    // Convert old ghost to new preview system
    commands.entity(preview_entity).insert((
        ToolPreview,
        ToolPreviewKind::Build {
            prototype_id: prototypes.active,
        },
        PlacementPreview { validation: None },
        NonInteractive,
    ));

    session.active = Some(ActiveToolSession::Build(BuildToolSession {
        prototype_id: prototypes.active,
        preview_entity,
        rotation_index: 0,
    }));
}

fn cleanup_build_session(
    mut commands: Commands,
    mut session: ResMut<ToolSessionState>,
) {
    crate::tools::cleanup_current_session(
        &mut commands,
        &mut session,
        crate::tools::ToolSessionEndReason::Replaced,
    );
}

pub fn build_tool_system(
    mut commands: Commands,
    pointer: Res<PointerContext>,
    gate: Res<ToolInputGate>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    mut tool: ResMut<ToolContext>,
    mut session: ResMut<ToolSessionState>,
    mut ghost_positions: Query<&mut WorldPos>,
    mut builds: MessageWriter<BuildObjectRequested>,
) {
    tool.sync_from_pointer(&pointer);

    if !gate.can_use_world() {
        return;
    }

    if gate.cancel_requested {
        crate::tools::cleanup_current_session(
            &mut commands,
            &mut session,
            crate::tools::ToolSessionEndReason::Cancelled,
        );
        next_mode.set(ToolMode::Cursor);
        return;
    }

    if let Some(ActiveToolSession::Build(build_session)) = session.active.as_ref() {
        if let Ok(mut world_pos) = ghost_positions.get_mut(build_session.preview_entity) {
            world_pos.0 = pointer.world_pos;
        }

        if gate.primary_click_released {
            // Re-check validity before committing
            builds.write(BuildObjectRequested {
                prototype: build_session.prototype_id,
                pos: pointer.world_pos,
                rotation: build_session.rotation_index,
            });
        }
    }
}
