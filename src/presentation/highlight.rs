use bevy::prelude::*;

use crate::objects::components::*;
use crate::tools::{ActiveToolAction, ToolContext, ToolMode};

pub fn update_highlight_intents(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    tool: Res<ToolContext>,
    highlighted: Query<Entity, With<HighlightIntent>>,
    movable: Query<(), With<Movable>>,
    deletable: Query<(), With<Deletable>>,
    ghost: Query<(), With<BuildGhost>>,
    selected: Query<Entity, With<Selected>>,
) {
    for entity in &highlighted {
        commands.entity(entity).remove::<HighlightIntent>();
    }

    let mut candidates: Vec<(Entity, HighlightKind)> = Vec::new();

    match &tool.active {
        Some(ActiveToolAction::Moving { entity, valid, .. }) => {
            candidates.push((
                *entity,
                if *valid {
                    HighlightKind::MoveValid
                } else {
                    HighlightKind::MoveInvalid
                },
            ));
        }
        Some(ActiveToolAction::Building { ghost, valid, .. }) => {
            candidates.push((
                *ghost,
                if *valid {
                    HighlightKind::BuildValid
                } else {
                    HighlightKind::BuildInvalid
                },
            ));
        }
        Some(ActiveToolAction::PendingDelete { entity }) => {
            candidates.push((*entity, HighlightKind::DeleteDanger));
        }
        None => {
            if let Some(entity) = tool.hovered {
                let kind = match mode.get() {
                    ToolMode::Move if movable.get(entity).is_ok() => Some(HighlightKind::Hover),
                    ToolMode::Delete if deletable.get(entity).is_ok() => {
                        Some(HighlightKind::DeleteDanger)
                    }
                    ToolMode::Build if ghost.get(entity).is_ok() => Some(HighlightKind::BuildValid),
                    ToolMode::Cursor => Some(HighlightKind::Hover),
                    _ => None,
                };

                if let Some(kind) = kind {
                    candidates.push((entity, kind));
                }
            }
        }
    }

    if let Some(entity) = tool.hovered {
        candidates.push((entity, HighlightKind::Hover));
    }
    for entity in &selected {
        candidates.push((entity, HighlightKind::Selected));
    }

    candidates.sort_by_key(|(_, kind)| std::cmp::Reverse(kind.priority()));
    candidates.dedup_by_key(|(entity, _)| *entity);

    for (entity, kind) in candidates {
        commands.entity(entity).insert(HighlightIntent { kind });
    }
}

pub fn update_highlight_visuals(
    mut query: Query<
        (Option<&HighlightIntent>, Option<&Selected>, &mut Sprite),
        Without<crate::presentation::FootprintOutlineSegment>,
    >,
) {
    for (highlight, selected, mut sprite) in &mut query {
        sprite.color = match highlight.map(|intent| intent.kind) {
            Some(HighlightKind::MoveInvalid | HighlightKind::BuildInvalid) => {
                Color::srgba(1.0, 0.32, 0.28, 0.82)
            }
            Some(HighlightKind::DeleteDanger) => Color::srgba(1.0, 0.20, 0.18, 0.90),
            Some(HighlightKind::MoveValid | HighlightKind::BuildValid) => {
                Color::srgba(0.45, 1.0, 0.55, 0.72)
            }
            Some(HighlightKind::Hover) => Color::srgb(1.0, 0.94, 0.62),
            Some(HighlightKind::Selected) => Color::srgb(0.62, 0.82, 1.0),
            None if selected.is_some() => Color::srgb(0.62, 0.82, 1.0),
            None => Color::WHITE,
        };
    }
}
