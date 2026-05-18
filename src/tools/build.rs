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
use bevy::ecs::system::SystemParam;

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

fn apply_select_build_prototype_requests(mut params: SelectBuildPrototypeParams) {
    for request in params.requests.read() {
        if !params
            .catalog
            .prototypes
            .contains_key(&request.prototype_id)
        {
            warn!(
                "Request to select unknown prototype: {:?}",
                request.prototype_id
            );
            continue;
        }

        params.selection.selected_prototype_id = Some(request.prototype_id.clone());

        if *params.current_mode.get() == ToolMode::Build {
            crate::tools::cleanup_current_session(
                &mut params.commands,
                &mut params.session,
                crate::tools::ToolSessionEndReason::Replaced,
            );
            spawn_build_session(
                &mut params.commands,
                &params.asset_server,
                &params.catalog,
                &params.selection,
                &params.pointer,
                &mut params.session,
            );
        } else {
            params.activation.write(ActivateToolRequested {
                mode: ToolMode::Build,
                kind: ToolActivationKind::Replace,
            });
        }
    }
}

#[derive(SystemParam)]
struct SelectBuildPrototypeParams<'w, 's> {
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
    catalog: Res<'w, ObjectCatalog>,
    pointer: Res<'w, PointerContext>,
    current_mode: Res<'w, State<ToolMode>>,
    requests: MessageReader<'w, 's, SelectBuildPrototypeRequested>,
    selection: ResMut<'w, BuildSelectionState>,
    activation: MessageWriter<'w, ActivateToolRequested>,
    session: ResMut<'w, ToolSessionState>,
}

fn start_build_session(mut params: BuildSessionParams) {
    spawn_build_session(
        &mut params.commands,
        &params.asset_server,
        &params.catalog,
        &params.selection,
        &params.pointer,
        &mut params.session,
    );
}

#[derive(SystemParam)]
struct BuildSessionParams<'w, 's> {
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
    catalog: Res<'w, ObjectCatalog>,
    selection: Res<'w, BuildSelectionState>,
    pointer: Res<'w, PointerContext>,
    session: ResMut<'w, ToolSessionState>,
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

pub fn build_tool_system(mut params: BuildToolParams) {
    params
        .tool
        .sync_from_pointer(&params.pointer, &params.targets);

    if !params.gate.can_use_world() {
        return;
    }

    if params.gate.cancel_requested {
        crate::tools::cleanup_current_session(
            &mut params.commands,
            &mut params.session,
            crate::tools::ToolSessionEndReason::Cancelled,
        );
        params.next_mode.set(ToolMode::Cursor);
        return;
    }

    if let Some(ActiveToolSession::Build(build_session)) = params.session.active.as_mut() {
        // Reset freshness once button is fully released (and not in the release frame itself)
        if !params.actions.pressed(InputAction::PrimaryClick)
            && !params.actions.just_released(InputAction::PrimaryClick)
        {
            build_session.awaiting_fresh_click = false;
        }

        if let Ok(mut world_pos) = params.ghost_positions.get_mut(build_session.preview_entity) {
            world_pos.0 = params.pointer.world_pos;
        }

        if params.gate.primary_world_click_released && !build_session.awaiting_fresh_click {
            params.builds.write(BuildObjectRequested {
                prototype: build_session.prototype_id.clone(),
                pos: params.pointer.world_pos,
                rotation: build_session.rotation_index,
            });
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct BuildToolParams<'w, 's> {
    commands: Commands<'w, 's>,
    pointer: Res<'w, PointerContext>,
    targets: Res<'w, PointerTargets>,
    gate: Res<'w, ToolInputGate>,
    actions: Res<'w, InputActionState>,
    next_mode: ResMut<'w, NextState<ToolMode>>,
    tool: ResMut<'w, ToolContext>,
    session: ResMut<'w, ToolSessionState>,
    ghost_positions: Query<'w, 's, &'static mut WorldPos>,
    builds: MessageWriter<'w, BuildObjectRequested>,
}
