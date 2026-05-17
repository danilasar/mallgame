use bevy::prelude::*;

use crate::objects::components::{BlocksPlacement, FootAnchor, Footprint, VisualOffset, WorldPos};
use crate::tools::{ActiveToolSession, ToolPreview, ToolSessionState, PreviewSource};

#[derive(Component, Debug, Clone)]
pub struct Rotatable {
    pub current: usize,
    pub variants: Vec<RotationVariant>,
}

#[derive(Debug, Clone)]
pub struct RotationVariant {
    pub sprite: Handle<Image>,
    pub footprint: Footprint,
    pub foot_anchor: Vec2,
    pub visual_offset: Vec2,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct RotateObjectRequested {
    pub entity: Entity,
    pub steps: i32,
}

pub struct ObjectRotationPlugin;

impl Plugin for ObjectRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RotateObjectRequested>().add_systems(
            Update,
            apply_rotate_requests.in_set(crate::tools::ToolSet::Commit),
        );
    }
}

pub fn apply_rotate_requests(
    mut requests: MessageReader<RotateObjectRequested>,
    mut session: ResMut<ToolSessionState>,
    world_bounds: Res<crate::store::WorldBounds>,
    store_area: Res<crate::store::StoreArea>,
    mut set: ParamSet<(
        Query<(Entity, &WorldPos, &Footprint, Option<&BlocksPlacement>)>,
        Query<(
            &mut Rotatable,
            &mut Sprite,
            &mut Footprint,
            &mut FootAnchor,
            &mut VisualOffset,
            &WorldPos,
            Option<&PreviewSource>,
        )>,
        Query<
            (&mut Sprite, &mut Footprint, &mut FootAnchor, &mut VisualOffset),
            (With<ToolPreview>, Without<Rotatable>),
        >,
    )>,
) {
    for request in requests.read() {
        // 1. Get rotation data and check if it's a preview
        let (new_index, variant, is_preview, source_preview_entity, world_pos_val) = {
            let mut q1 = set.p1();
            let Ok((rotatable, _, _, _, _, world_pos, preview_source)) = q1.get_mut(request.entity) else {
                continue;
            };

            let len = rotatable.variants.len();
            if len == 0 {
                continue;
            }

            let current = rotatable.current as i32;
            let new_index = (current + request.steps).rem_euclid(len as i32) as usize;
            let variant = rotatable.variants[new_index].clone();
            let is_preview = preview_source.is_some();
            let source_preview_entity = preview_source.map(|s| s.preview_entity);
            let world_pos_val = world_pos.0;
            
            (new_index, variant, is_preview, source_preview_entity, world_pos_val)
        };

        // 2. REVALIDATION: Check if rotation is allowed for non-preview rotations
        if !is_preview {
            let footprints = set.p0();
            let validation = crate::placement::validate_placement(
                &world_bounds,
                &store_area,
                &footprints,
                &variant.footprint,
                world_pos_val,
                crate::placement::PlacementValidationOptions {
                    ignore_entity: Some(request.entity),
                },
            );

            if validation.is_err() {
                warn!("Rotation REJECTED for entity={:?}: {:?}", request.entity, validation.err());
                continue;
            }
        }

        if is_preview {
            // DO NOT mutate real entity if it's being moved. Rotate the preview instead.
            if let Some(preview_entity) = source_preview_entity {
                if let Ok((mut p_sprite, mut p_fp, mut p_anchor, mut p_offset)) = set.p2().get_mut(preview_entity) {
                    p_sprite.image = variant.sprite;
                    *p_fp = variant.footprint;
                    p_anchor.0 = variant.foot_anchor;
                    p_offset.0 = variant.visual_offset;
                }
            }
            
            // Update session state so commit uses the new rotation
            if let Some(ActiveToolSession::Move(s)) = session.active.as_mut() {
                if s.source_entity == request.entity {
                    s.rotation_index = new_index;
                }
            } else if let Some(ActiveToolSession::Build(s)) = session.active.as_mut() {
                s.rotation_index = new_index;
            }
            
            info!(
                "RotateObjectRequested applied to PREVIEW entity={:?} new_variant={}",
                request.entity, new_index
            );
        } else {
            // Normal rotation for stationary objects
            if let Ok((mut rotatable, mut sprite, mut footprint, mut foot_anchor, mut visual_offset, _, _)) = set.p1().get_mut(request.entity) {
                rotatable.current = new_index;
                sprite.image = variant.sprite;
                *footprint = variant.footprint;
                foot_anchor.0 = variant.foot_anchor;
                visual_offset.0 = variant.visual_offset;
                
                info!(
                    "RotateObjectRequested applied to REAL entity={:?} current_variant={}",
                    request.entity, rotatable.current
                );
            }
        }
    }
}
