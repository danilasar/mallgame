use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerTargets};
use crate::objects::components::{
    FootAnchor, Footprint, InteractionRole, Movable, ProjectedPos, RuntimeOwned, RuntimeOwner,
    SortBias, SortLayer, StoreObject, VisualOffset, WorldPos,
};
use crate::tools::{
    ActiveToolSession, MoveObjectCommitted, MoveToolSession, NonInteractive, PlacementPreview,
    PreviewSource, StartMoveObjectRequested, ToolContext, ToolDescriptor, ToolInputGate, ToolMode,
    ToolPreview, ToolPreviewKind, ToolRegistry, ToolSessionState, ToolSet,
};
use bevy::ecs::system::SystemParam;

pub struct MoveToolPlugin;

impl Plugin for MoveToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolRegistry>();
        app.world_mut()
            .resource_mut::<ToolRegistry>()
            .register(ToolDescriptor {
                mode: ToolMode::Move,
                action: InputAction::ToolMove,
                label: "Move",
            });

        app.add_systems(
            Update,
            (apply_start_move_object_requests, move_tool_system)
                .chain()
                .run_if(in_state(ToolMode::Move))
                .in_set(ToolSet::ToolUpdate),
        )
        .add_systems(OnExit(ToolMode::Move), cleanup_move_session);
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct StartMoveObjectParams<'w, 's> {
    commands: Commands<'w, 's>,
    requests: MessageReader<'w, 's, StartMoveObjectRequested>,
    movable: Query<
        'w,
        's,
        (
            &'static WorldPos,
            &'static FootAnchor,
            &'static Footprint,
            &'static VisualOffset,
            &'static Sprite,
            &'static SortBias,
            Option<&'static crate::objects::rotation::Rotatable>,
        ),
        (With<Movable>, With<StoreObject>),
    >,
    session: ResMut<'w, ToolSessionState>,
}

pub fn apply_start_move_object_requests(mut params: StartMoveObjectParams) {
    for request in params.requests.read() {
        if let Ok((
            world_pos,
            foot_anchor,
            footprint,
            visual_offset,
            sprite,
            sort_bias,
            rotatable,
        )) = params.movable.get(request.entity)
        {
            crate::tools::cleanup_current_session(
                &mut params.commands,
                &mut params.session,
                crate::tools::ToolSessionEndReason::Replaced,
            );

            let rotation_index = rotatable.map_or(0, |r| r.current);

            let preview_entity = params
                .commands
                .spawn((
                    Sprite {
                        image: sprite.image.clone(),
                        custom_size: sprite.custom_size,
                        color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                        ..default()
                    },
                    *world_pos,
                    ProjectedPos::default(),
                    *foot_anchor,
                    *visual_offset,
                    SortLayer::DragPreview,
                    *sort_bias,
                    footprint.clone(),
                    ToolPreview,
                    ToolPreviewKind::Move {
                        source_entity: request.entity,
                    },
                    PlacementPreview { validation: None },
                    NonInteractive,
                    InteractionRole::ToolPreview,
                    RuntimeOwned {
                        owner: RuntimeOwner::ToolPreview,
                    },
                    Name::new(format!("MovePreview of {:?}", request.entity)),
                ))
                .id();

            params
                .commands
                .entity(request.entity)
                .insert(PreviewSource { preview_entity });

            params.session.active = Some(ActiveToolSession::Move(MoveToolSession {
                source_entity: request.entity,
                preview_entity,
                original_world_pos: world_pos.0,
                rotation_index,
                awaiting_fresh_click: true,
            }));
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct MoveToolParams<'w, 's> {
    commands: Commands<'w, 's>,
    pointer: Res<'w, PointerContext>,
    targets: Res<'w, PointerTargets>,
    gate: Res<'w, ToolInputGate>,
    actions: Res<'w, InputActionState>,
    movable: Query<'w, 's, Entity, (With<Movable>, With<StoreObject>)>,
    ghost_positions: Query<
        'w,
        's,
        (&'static mut WorldPos, &'static PlacementPreview),
        (With<ToolPreview>, Without<StoreObject>),
    >,
    session: ResMut<'w, ToolSessionState>,
    committed: MessageWriter<'w, MoveObjectCommitted>,
    requests: MessageWriter<'w, StartMoveObjectRequested>,
    tool: ResMut<'w, ToolContext>,
}

pub fn move_tool_system(mut params: MoveToolParams) {
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
        return;
    }

    if let Some(ActiveToolSession::Move(move_session)) = params.session.active.as_mut() {
        // Reset freshness once button is fully released (and not in the release frame itself)
        if !params.actions.pressed(InputAction::PrimaryClick)
            && !params.actions.just_released(InputAction::PrimaryClick)
        {
            move_session.awaiting_fresh_click = false;
        }

        if let Ok((mut world_pos, preview)) =
            params.ghost_positions.get_mut(move_session.preview_entity)
        {
            world_pos.0 = params.pointer.world_pos;

            if params.gate.primary_world_click_released && !move_session.awaiting_fresh_click {
                let is_valid = preview.validation.as_ref().is_some_and(|r| r.is_ok());
                if is_valid {
                    params.committed.write(MoveObjectCommitted {
                        entity: move_session.source_entity,
                        new_pos: world_pos.0,
                        rotation: move_session.rotation_index,
                    });
                }
                crate::tools::cleanup_current_session(
                    &mut params.commands,
                    &mut params.session,
                    if is_valid {
                        crate::tools::ToolSessionEndReason::Committed
                    } else {
                        crate::tools::ToolSessionEndReason::Cancelled
                    },
                );
            }
        }
        return;
    }

    // Start move via click
    if params.gate.primary_world_press_started
        && let Some(entity) = params
            .tool
            .hovered_entity
            .filter(|entity| params.movable.get(*entity).is_ok())
    {
        params.requests.write(StartMoveObjectRequested { entity });
    }
}

pub fn cleanup_move_session(mut commands: Commands, mut session: ResMut<ToolSessionState>) {
    crate::tools::cleanup_current_session(
        &mut commands,
        &mut session,
        crate::tools::ToolSessionEndReason::Replaced,
    );
}
