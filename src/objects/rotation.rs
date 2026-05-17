use bevy::prelude::*;

use crate::objects::components::{FootAnchor, Footprint, VisualOffset};

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
    mut query: Query<(
        &mut Rotatable,
        &mut Sprite,
        &mut Footprint,
        &mut FootAnchor,
        &mut VisualOffset,
        Option<&crate::tools::PreviewSource>,
    )>,
    mut preview_query: Query<(
        &mut Sprite,
        &mut Footprint,
        &mut FootAnchor,
        &mut VisualOffset,
    ), (With<crate::tools::ToolPreview>, Without<Rotatable>)>,
    mut session: ResMut<crate::tools::ToolSessionState>,
) {
    for request in requests.read() {
        let Ok((mut rotatable, mut sprite, mut footprint, mut foot_anchor, mut visual_offset, preview_source)) =
            query.get_mut(request.entity)
        else {
            continue;
        };

        let len = rotatable.variants.len();
        if len == 0 {
            continue;
        }

        let current = rotatable.current as i32;
        let new_index = (current + request.steps).rem_euclid(len as i32) as usize;
        
        // Clone variant info to avoid borrowing rotatable.variants while mutating rotatable.current
        let variant = rotatable.variants[new_index].clone();

        if let Some(source) = preview_source {
            // DO NOT mutate real entity if it's being moved. Rotate the preview instead.
            if let Ok((mut p_sprite, mut p_fp, mut p_anchor, mut p_offset)) = preview_query.get_mut(source.preview_entity) {
                p_sprite.image = variant.sprite;
                *p_fp = variant.footprint;
                p_anchor.0 = variant.foot_anchor;
                p_offset.0 = variant.visual_offset;
            }
            
            // Update session state so commit uses the new rotation
            if let Some(crate::tools::ActiveToolSession::Move(s)) = session.active.as_mut() {
                if s.source_entity == request.entity {
                    s.rotation_index = new_index;
                }
            } else if let Some(crate::tools::ActiveToolSession::Build(s)) = session.active.as_mut() {
                s.rotation_index = new_index;
            }
            
            info!(
                "RotateObjectRequested applied to PREVIEW entity={:?} new_variant={}",
                request.entity, new_index
            );
        } else {
            // Normal rotation for stationary objects
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
