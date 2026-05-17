use bevy::prelude::*;

use crate::input::{InputAction, InputActionState};
use crate::tools::{DeleteObjectRequested, ToolContext};
use crate::ui::{
    BlocksWorldInput, ModalLayer, UiSet,
    buttons::{label_text, ui_button},
};

#[derive(Resource, Debug, Default)]
pub struct ModalStack {
    pub stack: Vec<ModalInstance>,
    next_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModalId(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct ModalInstance {
    pub id: ModalId,
    pub kind: ModalKind,
    pub blocks_world: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ModalKind {
    ConfirmDelete { entity: Entity },
}

#[derive(Message, Debug, Clone, Copy)]
pub enum ModalRequest {
    Open(ModalKind),
    ConfirmTop,
    CancelTop,
}

#[derive(Component, Debug, Clone, Copy)]
struct ModalVisual;

#[derive(Component, Debug, Clone, Copy)]
enum ModalButton {
    Confirm,
    Cancel,
}

pub struct ModalUiPlugin;

impl Plugin for ModalUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModalStack>()
            .add_message::<ModalRequest>()
            .add_systems(
                Update,
                (
                    modal_keyboard_input_system,
                    modal_button_system,
                    apply_modal_requests,
                    render_modal_stack,
                )
                    .chain()
                    .in_set(UiSet::Modal),
            );
    }
}

pub fn modal_keyboard_input_system(
    actions: Res<InputActionState>,
    stack: Res<ModalStack>,
    mut requests: MessageWriter<ModalRequest>,
) {
    if stack.stack.is_empty() {
        return;
    }
    if actions.just_pressed(InputAction::Confirm) {
        requests.write(ModalRequest::ConfirmTop);
    } else if actions.just_pressed(InputAction::Cancel) {
        requests.write(ModalRequest::CancelTop);
    }
}

fn modal_button_system(
    mut query: Query<(&Interaction, &ModalButton), Changed<Interaction>>,
    mut requests: MessageWriter<ModalRequest>,
) {
    for (interaction, button) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        requests.write(match button {
            ModalButton::Confirm => ModalRequest::ConfirmTop,
            ModalButton::Cancel => ModalRequest::CancelTop,
        });
    }
}

pub fn apply_modal_requests(
    mut stack: ResMut<ModalStack>,
    mut tool: ResMut<ToolContext>,
    mut requests: MessageReader<ModalRequest>,
    mut deletes: MessageWriter<DeleteObjectRequested>,
    existing: Query<(), With<crate::objects::components::WorldPos>>,
) {
    for request in requests.read() {
        match *request {
            ModalRequest::Open(kind) => {
                let id = ModalId(stack.next_id);
                stack.next_id += 1;
                stack.stack.push(ModalInstance {
                    id,
                    kind,
                    blocks_world: true,
                });
            }
            ModalRequest::ConfirmTop => {
                if let Some(instance) = stack.stack.pop() {
                    match instance.kind {
                        ModalKind::ConfirmDelete { entity } => {
                            if existing.get(entity).is_ok() {
                                deletes.write(DeleteObjectRequested { entity });
                            }
                        }
                    }
                }
                tool.active = None;
            }
            ModalRequest::CancelTop => {
                stack.stack.pop();
                tool.active = None;
            }
        }
    }
}

fn render_modal_stack(
    mut commands: Commands,
    stack: Res<ModalStack>,
    layer: Query<Entity, With<ModalLayer>>,
    visuals: Query<Entity, With<ModalVisual>>,
) {
    if !stack.is_changed() {
        return;
    }

    for entity in &visuals {
        commands.entity(entity).despawn();
    }

    let Some(instance) = stack.stack.last() else {
        return;
    };
    let Some(layer) = layer.iter().next() else {
        return;
    };

    let overlay = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            BlocksWorldInput,
            Interaction::default(),
            ModalVisual,
            Name::new("ModalOverlay"),
        ))
        .id();
    commands.entity(layer).add_child(overlay);

    let dialog = commands
        .spawn((
            Node {
                width: Val::Px(320.0),
                height: Val::Px(168.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.12, 0.10)),
            BlocksWorldInput,
            Interaction::default(),
            ModalVisual,
            Name::new(format!("ModalDialog {:?}", instance.id)),
        ))
        .with_children(|parent| match instance.kind {
            ModalKind::ConfirmDelete { .. } => {
                parent.spawn((
                    Text::new("Удалить объект?"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.98, 0.92, 0.78)),
                ));
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((ui_button("Удалить", 110.0, 38.0), ModalButton::Confirm))
                            .with_child(label_text("Удалить"));
                        row.spawn((ui_button("Отмена", 110.0, 38.0), ModalButton::Cancel))
                            .with_child(label_text("Отмена"));
                    });
            }
        })
        .id();
    commands.entity(overlay).add_child(dialog);
}
