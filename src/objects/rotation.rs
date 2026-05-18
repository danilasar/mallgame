use bevy::prelude::*;

pub struct ObjectRotationPlugin;

impl Plugin for ObjectRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RotateObjectRequested>().add_systems(
            Update,
            handle_rotate_requests
                .in_set(crate::store::commands::DomainCommandSet::RequestToCommand),
        );
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct RotateObjectRequested {
    pub entity: Entity,
    pub steps: i32,
}

#[derive(Component, Debug, Clone)]
pub struct Rotatable {
    pub current: usize,
    pub variants: Vec<RotationVariant>,
}

#[derive(Debug, Clone)]
pub struct RotationVariant {
    pub sprite: Handle<Image>,
    pub footprint: crate::objects::components::Footprint,
    pub foot_anchor: Vec2,
    pub visual_offset: Vec2,
}

#[allow(clippy::type_complexity)]
pub fn handle_rotate_requests(
    mut requests: MessageReader<RotateObjectRequested>,
    mut queue: ResMut<crate::store::commands::DomainCommandQueue>,
    mut session: ResMut<crate::tools::ToolSessionState>,
    query: Query<(
        &Rotatable,
        &crate::objects::components::ObjectStableId,
        Option<&crate::tools::PreviewSource>,
    )>,
    mut preview_query: Query<
        (
            &mut Sprite,
            &mut crate::objects::components::Footprint,
            &mut crate::objects::components::FootAnchor,
            &mut crate::objects::components::VisualOffset,
        ),
        (With<crate::tools::ToolPreview>, Without<Rotatable>),
    >,
) {
    for request in requests.read() {
        let Ok((rotatable, stable_id, preview_source)) = query.get(request.entity) else {
            continue;
        };

        let len = rotatable.variants.len();
        if len == 0 {
            continue;
        }

        let current = rotatable.current as i32;
        let new_index = (current + request.steps).rem_euclid(len as i32) as usize;
        let variant = rotatable.variants[new_index].clone();

        if let Some(source) = preview_source {
            // Preview rotation remains immediate for responsive UI
            if let Ok((mut p_sprite, mut p_fp, mut p_anchor, mut p_offset)) =
                preview_query.get_mut(source.preview_entity)
            {
                p_sprite.image = variant.sprite;
                *p_fp = variant.footprint;
                p_anchor.0 = variant.foot_anchor;
                p_offset.0 = variant.visual_offset;
            }

            // Update session state so commit uses the new rotation
            if let Some(crate::tools::ActiveToolSession::Move(crate::tools::MoveToolSession::Floor(s))) =
                session.active.as_mut()
            {
                if s.source_entity == request.entity {
                    s.rotation_index = new_index;
                }
            } else if let Some(crate::tools::ActiveToolSession::Build(s)) = session.active.as_mut()
            {
                match s {
                    crate::tools::BuildToolSession::Floor(floor) => {
                        floor.rotation_index = new_index;
                    }
                    crate::tools::BuildToolSession::WallMounted(wall) => {
                        wall.rotation_index = new_index;
                    }
                }
            }
        } else {
            // Real rotation goes through command queue
            queue
                .commands
                .push_back(crate::store::commands::DomainCommand::RotateObject(
                    crate::store::commands::RotateObjectCommand {
                        object_id: stable_id.0,
                        from_rotation: rotatable.current,
                        to_rotation: new_index,
                    },
                ));
        }
    }
}
