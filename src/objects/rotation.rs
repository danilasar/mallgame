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
    )>,
) {
    for request in requests.read() {
        let Ok((mut rotatable, mut sprite, mut footprint, mut foot_anchor, mut visual_offset)) =
            query.get_mut(request.entity)
        else {
            continue;
        };

        let len = rotatable.variants.len();
        if len == 0 {
            continue;
        }

        let current = rotatable.current as i32;
        rotatable.current = (current + request.steps).rem_euclid(len as i32) as usize;
        let variant = &rotatable.variants[rotatable.current];

        sprite.image = variant.sprite.clone();
        *footprint = variant.footprint.clone();
        foot_anchor.0 = variant.foot_anchor;
        visual_offset.0 = variant.visual_offset;
        info!(
            "RotateObjectRequested applied entity={:?} current_variant={}",
            request.entity, rotatable.current
        );
    }
}
