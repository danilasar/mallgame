use bevy::prelude::*;

use crate::objects::components::{
    Footprint, InteractionRole, RuntimeOwned, RuntimeOwner, WorldPos,
};
use crate::objects::rotation::Rotatable;
use crate::placement::{polygon_bounds, world_polygon};
use crate::presentation::{IsoProjection, sync_visual_transform, world_to_iso};
use crate::tools::{
    ObjectActionKind, ObjectActionOrigin, ObjectActionRequested, PointerPressOwner,
    PrimaryPointerCycle, ToolMode,
};
use crate::ui::{
    BlocksWorldInput, UiSet, WorldWidgetsLayer,
    buttons::{UiFonts, label_text},
};

#[derive(Component, Debug, Clone, Copy)]
pub struct RotateWorldWidget {
    pub target: Entity,
}

pub struct WorldWidgetUiPlugin;

impl Plugin for WorldWidgetUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rotate_widget_button_system.in_set(UiSet::WorldWidgets),
        )
        .add_systems(
            PostUpdate,
            update_contextual_world_widgets.after(sync_visual_transform),
        );
    }
}

fn rotate_widget_button_system(
    mut query: Query<(&Interaction, &RotateWorldWidget), Changed<Interaction>>,
    mut actions: MessageWriter<ObjectActionRequested>,
    mut cycle: ResMut<PrimaryPointerCycle>,
) {
    for (interaction, widget) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // IMPORTANT: Ensure the press owner is set correctly to block tools from using this click
        cycle.owner = PointerPressOwner::WorldWidget;
        cycle.consumed = true;

        actions.write(ObjectActionRequested {
            entity: widget.target,
            action: ObjectActionKind::Rotate,
            origin: ObjectActionOrigin::WorldWidget,
        });
        info!("Rotate widget clicked for entity={:?}", widget.target);
    }
}

pub fn update_contextual_world_widgets(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    session: Res<crate::tools::ToolSessionState>,
    targets: Res<crate::input::PointerTargets>,
    projection: Res<IsoProjection>,
    mut widgets: Query<(Entity, &mut RotateWorldWidget, &mut Node, &Interaction)>,
    fonts: Res<UiFonts>,
    layer: Query<Entity, With<WorldWidgetsLayer>>,
    objects: Query<(&WorldPos, &Footprint, Option<&Rotatable>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    // 1. Determine target entity.
    // In Move mode, show Rotate button ONLY if NOT currently dragging.
    // Cursor mode NO LONGER shows the rotate button (as per user request).
    let target = if *mode.get() == ToolMode::Move && session.active.is_none() {
        if let Some(hovered) = targets.world_object {
            Some(hovered)
        } else if let Some((_, widget, _, _)) = widgets.iter().next() {
            // Flicker Fix: Maintain target if we are hovering the widget itself.
            if targets.world_widget.is_some() {
                Some(widget.target)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let Some(target) = target else {
        for (entity, _, _, _) in &mut widgets {
            commands.entity(entity).despawn();
        }
        return;
    };

    let Ok((world_pos, footprint, rotatable)) = objects.get(target) else {
        for (entity, _, _, _) in &mut widgets {
            commands.entity(entity).despawn();
        }
        return;
    };

    if rotatable.is_none() {
        for (entity, _, _, _) in &mut widgets {
            commands.entity(entity).despawn();
        }
        return;
    };

    let Some(layer) = layer.iter().next() else {
        return;
    };
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };

    let polygon = world_polygon(footprint, world_pos.0);
    let Some(bounds) = polygon_bounds(&polygon) else {
        return;
    };

    let anchor_world = Vec2::new(bounds.max.x, bounds.max.y);
    let projected = world_to_iso(anchor_world, *projection) + Vec2::new(22.0, -22.0);
    let Ok(viewport) =
        camera.world_to_viewport(camera_transform, Vec3::new(projected.x, projected.y, 0.0))
    else {
        return;
    };

    if let Some((_, mut widget, mut node, _)) = widgets.iter_mut().next() {
        widget.target = target;
        node.left = Val::Px(viewport.x - 17.0);
        node.top = Val::Px(viewport.y - 17.0);
        node.display = Display::Flex;
    } else {
        let button = commands
            .spawn((
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(viewport.x - 17.0),
                    top: Val::Px(viewport.y - 17.0),
                    width: Val::Px(34.0),
                    height: Val::Px(34.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(17.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.30, 0.18, 0.08)),
                BlocksWorldInput,
                RotateWorldWidget { target },
                InteractionRole::WorldWidget,
                RuntimeOwned {
                    owner: RuntimeOwner::WorldWidget,
                },
                Name::new("RotateWorldWidget"),
            ))
            .with_child(label_text("↻", &fonts))
            .id();
        commands.entity(layer).add_child(button);
    }
}
