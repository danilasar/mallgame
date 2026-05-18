use bevy::prelude::*;

use crate::input::{
    InputAction, InputActionState, PointerContext, PointerTargets, point_in_convex_quad,
};
use crate::objects::components::{
    InteractionRole, RuntimeOwned, RuntimeOwner, VisualOffset, WallAttachmentPoint, WorldPos,
};
use crate::objects::prototypes::{
    BuildSelectionState, ObjectCatalog, PlacementKind, SelectBuildPrototypeRequested,
    spawn_ghost_from_prototype, spawn_wall_mounted_preview,
};
use crate::presentation::IsoProjection;
use crate::store::{WallSurface, wall_surface_visual_offset, wall_surface_world_pos};
use crate::tools::{
    ActivateToolRequested, ActiveToolSession, BuildObjectRequested, BuildToolSession,
    FloorBuildSession, NonInteractive, PlacementPreview, ToolActivationKind, ToolContext,
    ToolDescriptor, ToolInputGate, ToolMode, ToolPreview, ToolPreviewKind, ToolRegistry,
    ToolSessionState, ToolSet, WallMountedBuildSession, WallMountedPreview,
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

    match proto.placement.kind {
        PlacementKind::Floor => {
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

            session.active = Some(ActiveToolSession::Build(BuildToolSession::Floor(
                FloorBuildSession {
                    prototype_id,
                    preview_entity,
                    rotation_index: 0,
                    awaiting_fresh_click: true,
                },
            )));
        }
        PlacementKind::WallMounted => {
            let preview_entity = spawn_wall_mounted_preview(
                commands,
                asset_server,
                proto,
                pointer.world_pos,
                Vec2::ZERO,
                true,
            );

            session.active = Some(ActiveToolSession::Build(BuildToolSession::WallMounted(
                WallMountedBuildSession {
                    prototype_id,
                    preview_entity,
                    current_attachment: None,
                    rotation_index: 0,
                    awaiting_fresh_click: true,
                },
            )));
        }
    }
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
        match build_session {
            BuildToolSession::Floor(floor) => {
                if !params.actions.pressed(InputAction::PrimaryClick)
                    && !params.actions.just_released(InputAction::PrimaryClick)
                {
                    floor.awaiting_fresh_click = false;
                }

                if let Ok(mut world_pos) = params.ghost_positions.get_mut(floor.preview_entity) {
                    world_pos.0 = params.pointer.world_pos;
                }

                if params.gate.primary_world_click_released && !floor.awaiting_fresh_click {
                    params.builds.write(BuildObjectRequested {
                        prototype: floor.prototype_id.clone(),
                        placement: crate::objects::components::ObjectPlacement::Floor {
                            world_pos: params.pointer.world_pos,
                            rotation_index: Some(floor.rotation_index),
                        },
                    });
                }
            }
            BuildToolSession::WallMounted(wall) => {
                if !params.actions.pressed(InputAction::PrimaryClick)
                    && !params.actions.just_released(InputAction::PrimaryClick)
                {
                    wall.awaiting_fresh_click = false;
                }

                let preview_size = params
                    .wall_preview_size
                    .get(wall.preview_entity)
                    .ok()
                    .and_then(|sprite| sprite.custom_size)
                    .unwrap_or(Vec2::new(64.0, 64.0));
                let attachment = find_wall_attachment_candidate(
                    params.pointer.projected_pos,
                    *params.projection,
                    &params.wall_surfaces,
                    preview_size.x * 0.5,
                );

                wall.current_attachment = attachment.map(|(attachment, _, _)| attachment);

                if let Ok((mut world_pos, mut visual_offset, mut visibility, mut preview)) =
                    params.wall_previews.get_mut(wall.preview_entity)
                {
                    if let Some((_, base_world_pos, offset)) = attachment {
                        world_pos.0 = base_world_pos;
                        visual_offset.0 = offset;
                        *visibility = Visibility::Visible;
                        preview.validation = Some(Ok(()));
                    } else {
                        world_pos.0 = params.pointer.world_pos;
                        visual_offset.0 = Vec2::ZERO;
                        *visibility = Visibility::Visible;
                        preview.validation = Some(Err(
                            crate::store::PlacementInvalidReason::WallSurfaceMissing,
                        ));
                    }
                }

                if params.gate.primary_world_click_released
                    && !wall.awaiting_fresh_click
                    && let Some(attachment) = wall.current_attachment
                {
                    params.builds.write(BuildObjectRequested {
                        prototype: wall.prototype_id.clone(),
                        placement: crate::objects::components::ObjectPlacement::WallMounted {
                            attachment,
                        },
                    });
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct BuildToolParams<'w, 's> {
    commands: Commands<'w, 's>,
    pointer: Res<'w, PointerContext>,
    projection: Res<'w, IsoProjection>,
    targets: Res<'w, PointerTargets>,
    gate: Res<'w, ToolInputGate>,
    actions: Res<'w, InputActionState>,
    next_mode: ResMut<'w, NextState<ToolMode>>,
    tool: ResMut<'w, ToolContext>,
    session: ResMut<'w, ToolSessionState>,
    ghost_positions: Query<'w, 's, &'static mut WorldPos, Without<WallMountedPreview>>,
    wall_preview_size: Query<'w, 's, &'static Sprite, With<WallMountedPreview>>,
    wall_previews: Query<
        'w,
        's,
        (
            &'static mut WorldPos,
            &'static mut VisualOffset,
            &'static mut Visibility,
            &'static mut PlacementPreview,
        ),
        With<WallMountedPreview>,
    >,
    wall_surfaces: Query<'w, 's, (Entity, &'static WallSurface)>,
    builds: MessageWriter<'w, BuildObjectRequested>,
}

#[allow(dead_code)]
pub fn wall_attachment_from_hit(
    hit: crate::input::WallSurfaceHit,
    preview_half_width: f32,
    wall_length: f32,
    wall_height: f32,
) -> WallAttachmentPoint {
    let max_offset = (wall_length - preview_half_width).max(preview_half_width);
    let offset_along_segment = hit
        .offset_along_segment
        .clamp(preview_half_width, max_offset);
    let height_on_wall = hit.height_on_wall.clamp(0.0, wall_height);

    WallAttachmentPoint {
        segment_key: hit.key,
        offset_along_segment,
        height_on_wall,
    }
}

pub fn find_wall_attachment_candidate(
    projected_pos: Vec2,
    projection: IsoProjection,
    wall_surfaces: &Query<(Entity, &WallSurface)>,
    preview_half_width: f32,
) -> Option<(WallAttachmentPoint, Vec2, Vec2)> {
    let mut best: Option<(f32, WallAttachmentPoint, Vec2, Vec2)> = None;

    for (_entity, surface) in wall_surfaces.iter() {
        let projected_start = crate::presentation::world_to_iso(surface.start, projection);
        let projected_end = crate::presentation::world_to_iso(surface.end, projection);
        let segment = projected_end - projected_start;
        if segment.length_squared() <= f32::EPSILON {
            continue;
        }

        let segment_length = segment.length();
        let wall_direction = segment / segment_length;
        let wall_normal = Vec2::new(-wall_direction.y, wall_direction.x);
        let thickness_offset = wall_normal * surface.thickness;
        let quad = [
            projected_start,
            projected_end,
            projected_end + thickness_offset + Vec2::new(0.0, surface.height),
            projected_start + thickness_offset + Vec2::new(0.0, surface.height),
        ];

        let relative = projected_pos - projected_start;
        let along = relative.dot(wall_direction).clamp(0.0, segment_length);
        let base_projected = projected_start + wall_direction * along;
        let surface_base_projected = base_projected + thickness_offset;
        let t = (along / segment_length).clamp(0.0, 1.0);
        let offset_along_segment = (surface.length * t).clamp(preview_half_width, surface.length);
        let height_on_wall =
            (projected_pos.y - surface_base_projected.y).clamp(0.0, surface.height);

        // Distance from the plane of the wall surface
        // projected_pos is somewhere. surface_base_projected is the bottom edge of the wall face.
        let to_surface_base = projected_pos - surface_base_projected;
        let distance_from_wall_plane = to_surface_base.dot(wall_normal).abs();

        let inside_quad = point_in_convex_quad(projected_pos, quad);
        
        let acceptance = if inside_quad {
            distance_from_wall_plane
        } else {
            // If outside, penalize by actual distance to the quad
            projected_pos.distance(base_projected) + 10_000.0
        };

        if acceptance > surface.thickness + preview_half_width + 48.0 {
            continue;
        }
        let attachment = WallAttachmentPoint {
            segment_key: surface.key,
            offset_along_segment,
            height_on_wall,
        };
        let base_world = wall_surface_world_pos(surface, offset_along_segment);
        let visual_offset = wall_surface_visual_offset(surface, projection, height_on_wall);

        if best
            .as_ref()
            .is_none_or(|(best_acceptance, _, _, _)| acceptance < *best_acceptance)
        {
            best = Some((acceptance, attachment, base_world, visual_offset));
        }
    }

    best.map(|(_, attachment, base_world, visual_offset)| (attachment, base_world, visual_offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::WallSurfaceHit;
    use crate::store::{StoreBoundarySide, StoreChunkCoord, WallSegmentKey};

    #[test]
    fn wall_attachment_from_hit_clamps_to_surface_bounds() {
        let hit = WallSurfaceHit {
            entity: Entity::from_bits(1),
            key: WallSegmentKey {
                chunk: StoreChunkCoord { x: 0, y: 0 },
                side: StoreBoundarySide::Top,
            },
            world_pos: Vec2::new(3.0, 7.0),
            offset_along_segment: 0.2,
            height_on_wall: 20.0,
            normal: Vec2::Y,
        };

        let attachment = wall_attachment_from_hit(hit, 1.0, 4.0, 6.0);

        assert_eq!(attachment.segment_key, hit.key);
        assert!((attachment.offset_along_segment - 1.0).abs() < 0.001);
        assert!((attachment.height_on_wall - 6.0).abs() < 0.001);
    }
}
