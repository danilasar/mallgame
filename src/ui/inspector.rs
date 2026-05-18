use bevy::prelude::*;

use crate::objects::components::{Deletable, Movable};
use crate::objects::rotation::Rotatable;
use crate::tools::{
    ObjectActionKind, ObjectActionOrigin, ObjectActionRequested, SelectionState, ToolMode,
};
use crate::ui::buttons::{UiFonts, label_text, ui_button};
use crate::ui::{ActiveInterfacePanel, InterfacePanelId};

#[derive(Component, Debug, Clone, Copy)]
pub struct InspectorActionButton {
    pub entity: Entity,
    pub action: ObjectActionKind,
}

#[allow(clippy::type_complexity)]
pub fn render_object_inspector(
    commands: &mut Commands,
    parent: Entity,
    selection: &SelectionState,
    fonts: &UiFonts,
    mode: ToolMode,
    objects: Query<(
        Option<&Name>,
        Option<&Movable>,
        Option<&Rotatable>,
        Option<&Deletable>,
    )>,
) {
    let Some(target) = selection.primary else {
        commands
            .entity(parent)
            .with_child(label_text("No selection", fonts));
        return;
    };

    let Ok((name, movable, rotatable, deletable)) = objects.get(target) else {
        commands
            .entity(parent)
            .with_child(label_text("Stale selection", fonts));
        return;
    };

    let name_str = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Object".to_string());
    let id_str = format!("ID: {:?}", target);

    commands.entity(parent).with_children(|ui| {
        ui.spawn(label_text(name_str, fonts));
        ui.spawn(label_text(id_str, fonts));

        if movable.is_some() {
            ui.spawn((
                ui_button("Move", 120.0, 30.0),
                InspectorActionButton {
                    entity: target,
                    action: ObjectActionKind::Move,
                },
            ))
            .with_child(label_text("Move", fonts));
        }

        if rotatable.is_some() && mode == ToolMode::Move {
            ui.spawn((
                ui_button("Rotate", 120.0, 30.0),
                InspectorActionButton {
                    entity: target,
                    action: ObjectActionKind::Rotate,
                },
            ))
            .with_child(label_text("Rotate", fonts));
        }

        if deletable.is_some() {
            ui.spawn((
                ui_button("Delete", 120.0, 30.0),
                InspectorActionButton {
                    entity: target,
                    action: ObjectActionKind::Delete,
                },
            ))
            .with_child(label_text("Delete", fonts));
        }
    });
}

pub fn inspector_button_system(
    mut query: Query<(&Interaction, &InspectorActionButton), Changed<Interaction>>,
    mut actions: MessageWriter<ObjectActionRequested>,
) {
    for (interaction, button) in &mut query {
        if *interaction == Interaction::Pressed {
            actions.write(ObjectActionRequested {
                entity: button.entity,
                action: button.action,
                origin: ObjectActionOrigin::InspectorButton,
            });
        }
    }
}

pub fn auto_open_inspector_system(
    selection: Res<SelectionState>,
    mut active_panel: ResMut<ActiveInterfacePanel>,
) {
    if selection.is_changed() && selection.primary.is_some() {
        active_panel.id = Some(InterfacePanelId::ObjectInspector);
    }
}
