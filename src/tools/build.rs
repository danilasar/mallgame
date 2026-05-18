use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerTargets};
use crate::objects::components::{InteractionRole, RuntimeOwned, RuntimeOwner, WorldPos};
use crate::objects::prototypes::{
    BuildSelectionState, ObjectCatalog, SelectBuildPrototypeRequested, spawn_ghost_from_prototype,
};
use crate::tools::{
    ActivateToolRequested, ActiveToolSession, BuildObjectRequested, BuildToolSession,
    NonInteractive, PlacementPreview, ToolActivationKind, ToolContext, ToolDescriptor,
    ToolInputGate, ToolMode, ToolPreview, ToolPreviewKind, ToolRegistry, ToolSessionState, ToolSet,
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

        app.init_resource::<BuildSelectionState>()
            .add_message::<SelectBuildPrototypeRequested>()
            .add_systems(OnEnter(ToolMode::Build), start_build_session)
            .add_systems(OnExit(ToolMode::Build), cleanup_build_session)
            .add_systems(
                Update,
                (
                    apply_select_build_prototype_requests,
                    build_tool_system.run_if(in_state(ToolMode::Build)),
                )
                    .chain()
                    .in_set(ToolSet::ToolUpdate),
            );
    }
}

fn apply_select_build_prototype_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<ObjectCatalog>,
    pointer: Res<PointerContext>,
    current_mode: Res<State<ToolMode>>,
    mut requests: MessageReader<SelectBuildPrototypeRequested>,
    mut selection: ResMut<BuildSelectionState>,
    mut activation: MessageWriter<ActivateToolRequested>,
    mut session: ResMut<ToolSessionState>,
) {
    for request in requests.read() {
        if !catalog.prototypes.contains_key(&request.prototype_id) {
            warn!(
                "Request to select unknown prototype: {:?}",
                request.prototype_id
            );
            continue;
        }

        selection.selected_prototype_id = Some(request.prototype_id.clone());

        if *current_mode.get() == ToolMode::Build {
            crate::tools::cleanup_current_session(
                &mut commands,
                &mut session,
                crate::tools::ToolSessionEndReason::Replaced,
            );
            spawn_build_session(
                &mut commands,
                &asset_server,
                &catalog,
                &selection,
                &pointer,
                &mut session,
            );
        } else {
            activation.write(ActivateToolRequested {
                mode: ToolMode::Build,
                kind: ToolActivationKind::Replace,
            });
        }
    }
}

fn start_build_session(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<ObjectCatalog>,
    selection: Res<BuildSelectionState>,
    pointer: Res<PointerContext>,
    mut session: ResMut<ToolSessionState>,
) {
    spawn_build_session(
        &mut commands,
        &asset_server,
        &catalog,
        &selection,
        &pointer,
        &mut session,
    );
}

fn spawn_build_session(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &ObjectCatalog,
    selection: &BuildSelectionState,
    pointer: &PointerContext,
    session: &mut ToolSessionState,
) {
    let Some(prototype_id) = selection.selected_prototype_id.clone() else {
        warn!("Cannot start build session: no prototype selected");
        return;
    };

    let Some(proto) = catalog.prototypes.get(&prototype_id) else {
        warn!(
            "Cannot start build session: unknown prototype {:?}",
            prototype_id
        );
        return;
    };

    let preview_entity =
        spawn_ghost_from_prototype(commands, asset_server, proto, pointer.world_pos);

    commands.entity(preview_entity).insert((
        ToolPreview,
        ToolPreviewKind::Build {
            prototype_id: prototype_id.clone(),
        },
        PlacementPreview { validation: None },
        NonInteractive,
        InteractionRole::ToolPreview,
        RuntimeOwned {
            owner: RuntimeOwner::ToolPreview,
        },
    ));

    session.active = Some(ActiveToolSession::Build(BuildToolSession {
        prototype_id,
        preview_entity,
        rotation_index: 0,
        awaiting_fresh_click: true,
    }));
}

fn cleanup_build_session(mut commands: Commands, mut session: ResMut<ToolSessionState>) {
    crate::tools::cleanup_current_session(
        &mut commands,
        &mut session,
        crate::tools::ToolSessionEndReason::Replaced,
    );
}

pub fn build_tool_system(
    mut commands: Commands,
    pointer: Res<PointerContext>,
    targets: Res<PointerTargets>,
    gate: Res<ToolInputGate>,
    actions: Res<InputActionState>,
    mut next_mode: ResMut<NextState<ToolMode>>,
    mut tool: ResMut<ToolContext>,
    mut session: ResMut<ToolSessionState>,
    mut ghost_positions: Query<&mut WorldPos>,
    mut builds: MessageWriter<BuildObjectRequested>,
) {
    tool.sync_from_pointer(&pointer, &targets);

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

    if let Some(ActiveToolSession::Build(build_session)) = session.active.as_mut() {
        // Reset freshness once button is fully released (and not in the release frame itself)
        if !actions.pressed(InputAction::PrimaryClick)
            && !actions.just_released(InputAction::PrimaryClick)
        {
            build_session.awaiting_fresh_click = false;
        }

        if let Ok(mut world_pos) = ghost_positions.get_mut(build_session.preview_entity) {
            world_pos.0 = pointer.world_pos;
        }

        if gate.primary_world_click_released && !build_session.awaiting_fresh_click {
            builds.write(BuildObjectRequested {
                prototype: build_session.prototype_id.clone(),
                pos: pointer.world_pos,
                rotation: build_session.rotation_index,
            });
        }
    }
}
