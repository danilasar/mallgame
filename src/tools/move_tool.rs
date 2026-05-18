use bevy::prelude::*;

use crate::input::{InputAction, InputActionState, PointerContext, PointerTargets};
use crate::objects::components::{
    FootAnchor, Footprint, InteractionRole, Movable, ObjectPlacement, ObjectPrototypeId,
    ProjectedPos, RuntimeOwned, RuntimeOwner, SortBias, SortLayer, StoreObject, VisualOffset,
    WallMountedPlacement, WallMovable, WallMovePreview, WorldPos,
};
use crate::tools::{
    ActiveToolSession, FloorMoveSession, MoveObjectCommitted, MoveToolSession, NonInteractive,
    PlacementPreview, PreviewSource, StartMoveObjectRequested, ToolContext, ToolDescriptor,
    ToolInputGate, ToolMode, ToolPreview, ToolPreviewKind, ToolRegistry, ToolSessionState, ToolSet,
    WallMoveSession,
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
    movable_floor: Query<
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
    movable_wall: Query<
        'w,
        's,
        (
            &'static WallMountedPlacement,
            &'static ObjectPrototypeId,
            &'static VisualOffset,
            &'static Sprite,
            &'static SortBias,
            Option<&'static crate::objects::rotation::Rotatable>,
        ),
        (With<WallMovable>, With<StoreObject>),
    >,
    movable_door: Query<
        'w,
        's,
        (
            &'static WallMountedPlacement,
            &'static ObjectPrototypeId,
            &'static VisualOffset,
            &'static Sprite,
            &'static SortBias,
        ),
        (
            With<crate::objects::components::Doorway>,
            With<crate::objects::components::DoorMovable>,
            With<StoreObject>,
        ),
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
        )) = params.movable_floor.get(request.entity)
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

            params.session.active = Some(ActiveToolSession::Move(MoveToolSession::Floor(
                FloorMoveSession {
                    source_entity: request.entity,
                    preview_entity,
                    original_world_pos: world_pos.0,
                    rotation_index,
                    awaiting_fresh_click: true,
                },
            )));
        } else if let Ok((placement, prototype_id, visual_offset, sprite, sort_bias)) =
            params.movable_door.get(request.entity)
        {
            crate::tools::cleanup_current_session(
                &mut params.commands,
                &mut params.session,
                crate::tools::ToolSessionEndReason::Replaced,
            );

            let preview_entity = params
                .commands
                .spawn((
                    Sprite {
                        image: sprite.image.clone(),
                        custom_size: sprite.custom_size,
                        color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                        ..default()
                    },
                    WorldPos::default(),
                    ProjectedPos::default(),
                    FootAnchor::default(),
                    *visual_offset,
                    SortLayer::DragPreview,
                    *sort_bias,
                    ToolPreview,
                    ToolPreviewKind::Move {
                        source_entity: request.entity,
                    },
                    PlacementPreview { validation: None },
                    NonInteractive,
                    InteractionRole::ToolPreview,
                    WallMovePreview,
                    RuntimeOwned {
                        owner: RuntimeOwner::ToolPreview,
                    },
                    Name::new(format!("DoorMovePreview of {:?}", request.entity)),
                ))
                .id();

            let access_zone_preview_entity = params
                .commands
                .spawn((
                    crate::objects::components::DoorAccessZonePreview,
                    ToolPreview,
                    NonInteractive,
                    Visibility::Hidden,
                    RuntimeOwned {
                        owner: RuntimeOwner::ToolPreview,
                    },
                    Name::new("DoorMoveAccessZonePreview"),
                ))
                .id();

            params
                .commands
                .entity(request.entity)
                .insert(PreviewSource { preview_entity });

            params.session.active = Some(ActiveToolSession::Move(MoveToolSession::Door(
                crate::tools::DoorMoveSession {
                    source_entity: request.entity,
                    preview_entity,
                    access_zone_preview_entity: Some(access_zone_preview_entity),
                    prototype_id: prototype_id.0.clone(),
                    original_attachment: placement.attachment,
                    current_attachment: None,
                    current_derived: None,
                    awaiting_fresh_click: true,
                },
            )));
        } else if let Ok((placement, prototype_id, visual_offset, sprite, sort_bias, rotatable)) =
            params.movable_wall.get(request.entity)
        {
            crate::tools::cleanup_current_session(
                &mut params.commands,
                &mut params.session,
                crate::tools::ToolSessionEndReason::Replaced,
            );

            let _rotation_index = rotatable.map_or(0, |r| r.current);

            let preview_entity = params
                .commands
                .spawn((
                    Sprite {
                        image: sprite.image.clone(),
                        custom_size: sprite.custom_size,
                        color: Color::srgba(0.65, 0.90, 1.0, 0.55),
                        ..default()
                    },
                    WorldPos::default(),
                    ProjectedPos::default(),
                    FootAnchor::default(),
                    *visual_offset,
                    SortLayer::DragPreview,
                    *sort_bias,
                    ToolPreview,
                    ToolPreviewKind::Move {
                        source_entity: request.entity,
                    },
                    PlacementPreview { validation: None },
                    NonInteractive,
                    InteractionRole::ToolPreview,
                    WallMovePreview,
                    RuntimeOwned {
                        owner: RuntimeOwner::ToolPreview,
                    },
                    Name::new(format!("WallMovePreview of {:?}", request.entity)),
                ))
                .id();

            params
                .commands
                .entity(request.entity)
                .insert(PreviewSource { preview_entity });

            params.session.active = Some(ActiveToolSession::Move(MoveToolSession::WallMounted(
                WallMoveSession {
                    source_entity: request.entity,
                    preview_entity,
                    prototype_id: prototype_id.0.clone(),
                    original_attachment: placement.attachment,
                    current_attachment: None,
                    awaiting_fresh_click: true,
                },
            )));
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
    movable_floor: Query<'w, 's, Entity, (With<Movable>, With<StoreObject>)>,
    movable_wall: Query<'w, 's, Entity, (With<WallMovable>, With<StoreObject>)>,
    movable_door: Query<
        'w,
        's,
        Entity,
        (
            With<crate::objects::components::Doorway>,
            With<crate::objects::components::DoorMovable>,
            With<StoreObject>,
        ),
    >,
    ghost_positions: Query<
        'w,
        's,
        (
            &'static mut WorldPos,
            &'static mut ProjectedPos,
            &'static mut FootAnchor,
            &'static mut VisualOffset,
            &'static mut PlacementPreview,
        ),
        (With<ToolPreview>, Without<StoreObject>),
    >,
    session: ResMut<'w, ToolSessionState>,
    committed: MessageWriter<'w, MoveObjectCommitted>,
    requests: MessageWriter<'w, StartMoveObjectRequested>,
    tool: ResMut<'w, ToolContext>,
    catalog: Res<'w, crate::objects::prototypes::ObjectCatalog>,
    wall_surfaces: Query<'w, 's, (Entity, &'static crate::store::WallSurface)>,
    projection: Res<'w, crate::presentation::IsoProjection>,
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
        match move_session {
            MoveToolSession::Floor(floor_session) => {
                if !params.actions.pressed(InputAction::PrimaryClick)
                    && !params.actions.just_released(InputAction::PrimaryClick)
                {
                    floor_session.awaiting_fresh_click = false;
                }

                if let Ok((mut world_pos, _, _, _, preview)) =
                    params.ghost_positions.get_mut(floor_session.preview_entity)
                {
                    world_pos.0 = params.pointer.world_pos;

                    if params.gate.primary_world_click_released
                        && !floor_session.awaiting_fresh_click
                    {
                        let is_valid = preview.validation.as_ref().is_some_and(|r| r.is_ok());
                        if is_valid {
                            params.committed.write(MoveObjectCommitted {
                                entity: floor_session.source_entity,
                                new_placement: ObjectPlacement::Floor {
                                    world_pos: world_pos.0,
                                    rotation_index: Some(floor_session.rotation_index),
                                },
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
            }
            MoveToolSession::WallMounted(wall_session) => {
                if !params.actions.pressed(InputAction::PrimaryClick)
                    && !params.actions.just_released(InputAction::PrimaryClick)
                {
                    wall_session.awaiting_fresh_click = false;
                }

                if let Ok((
                    mut world_pos,
                    mut projected_pos,
                    _foot_anchor,
                    mut visual_offset,
                    mut preview,
                )) = params.ghost_positions.get_mut(wall_session.preview_entity)
                {
                    // Update preview placement
                    let proto = params.catalog.prototypes.get(&wall_session.prototype_id);
                    let wall_spec = proto.and_then(crate::objects::prototypes::wall_mounted_spec);

                    if let Some(spec) = wall_spec {
                        let attachment = crate::tools::build::find_wall_attachment_candidate(
                            params.pointer.projected_pos,
                            *params.projection,
                            &params.wall_surfaces,
                            spec.width * 0.5,
                        );

                        if let Some((valid_attachment, v_pos, _)) = attachment {
                            world_pos.0 = v_pos;
                            visual_offset.0 = crate::store::wall_surface_visual_offset(
                                params
                                    .wall_surfaces
                                    .iter()
                                    .find(|(_, s)| s.key == valid_attachment.segment_key)
                                    .unwrap()
                                    .1,
                                *params.projection,
                                valid_attachment.height_on_wall,
                            );
                            projected_pos.0 =
                                crate::presentation::world_to_iso(v_pos, *params.projection);
                            wall_session.current_attachment = Some(valid_attachment);
                            preview.validation = Some(Ok(()));
                        } else {
                            wall_session.current_attachment = None;
                            preview.validation = None;
                        }
                    } else {
                        wall_session.current_attachment = None;
                        preview.validation = None;
                    }

                    // Commit
                    if params.gate.primary_world_click_released
                        && !wall_session.awaiting_fresh_click
                    {
                        let is_valid = preview.validation.as_ref().is_some_and(|r| r.is_ok());
                        if is_valid && let Some(attachment) = wall_session.current_attachment {
                            params.committed.write(MoveObjectCommitted {
                                entity: wall_session.source_entity,
                                new_placement: ObjectPlacement::WallMounted { attachment },
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
            }
            MoveToolSession::Door(door_session) => {
                if !params.actions.pressed(InputAction::PrimaryClick)
                    && !params.actions.just_released(InputAction::PrimaryClick)
                {
                    door_session.awaiting_fresh_click = false;
                }

                if let Ok((
                    mut world_pos,
                    mut projected_pos,
                    _foot_anchor,
                    mut visual_offset,
                    mut preview,
                )) = params.ghost_positions.get_mut(door_session.preview_entity)
                {
                    let proto = params.catalog.prototypes.get(&door_session.prototype_id);
                    let wall_spec = proto.and_then(crate::objects::prototypes::wall_mounted_spec);
                    let doorway_spec = proto.and_then(crate::objects::prototypes::doorway_spec);

                    if let Some(spec) = wall_spec
                        && let Some(door_spec) = doorway_spec
                    {
                        let attachment = crate::tools::build::find_wall_attachment_candidate(
                            params.pointer.projected_pos,
                            *params.projection,
                            &params.wall_surfaces,
                            spec.width * 0.5,
                        );

                        if let Some((raw_attachment, v_pos, _)) = attachment {
                            let valid_attachment =
                                crate::objects::prototypes::normalize_wall_attachment_for_prototype(
                                    proto.unwrap(),
                                    raw_attachment,
                                );
                            let surface = params
                                .wall_surfaces
                                .iter()
                                .find(|(_, s)| s.key == valid_attachment.segment_key)
                                .unwrap()
                                .1;
                            world_pos.0 = v_pos;
                            visual_offset.0 = crate::store::wall_surface_visual_offset(
                                surface,
                                *params.projection,
                                valid_attachment.height_on_wall,
                            );
                            projected_pos.0 =
                                crate::presentation::world_to_iso(v_pos, *params.projection);
                            door_session.current_attachment = Some(valid_attachment);

                            // Derive door placement for validation
                            if let Ok(derived) = crate::store::boundary::derive_door_placement(
                                spec.width,
                                spec.height,
                                door_spec.access_width,
                                door_spec.access_depth,
                                valid_attachment,
                                surface,
                                crate::objects::prototypes::wall_occupancy_kind_for_prototype(
                                    proto.unwrap(),
                                ),
                            ) {
                                door_session.current_derived = Some(derived);
                            } else {
                                door_session.current_derived = None;
                            }

                            preview.validation = Some(Ok(()));
                        } else {
                            door_session.current_attachment = None;
                            door_session.current_derived = None;
                            preview.validation = None;
                        }
                    } else {
                        door_session.current_attachment = None;
                        door_session.current_derived = None;
                        preview.validation = None;
                    }

                    // Commit
                    if params.gate.primary_world_click_released
                        && !door_session.awaiting_fresh_click
                    {
                        let is_valid = preview.validation.as_ref().is_some_and(|r| r.is_ok());
                        if is_valid && let Some(attachment) = door_session.current_attachment {
                            params.committed.write(MoveObjectCommitted {
                                entity: door_session.source_entity,
                                new_placement: ObjectPlacement::WallMounted { attachment },
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
            }
        }
    }

    // Start move via click
    if params.session.active.is_none()
        && params.gate.primary_world_press_started
        && let Some(entity) = params.tool.hovered_entity
        && (params.movable_floor.get(entity).is_ok()
            || params.movable_wall.get(entity).is_ok()
            || params.movable_door.get(entity).is_ok())
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
