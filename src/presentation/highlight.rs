use bevy::prelude::*;

use crate::objects::components::*;
use crate::tools::{PlacementPreview, PreviewSource, SelectionState, ToolContext, ToolMode};

pub fn update_highlight_intents(
    mut commands: Commands,
    mode: Res<State<ToolMode>>,
    tool: Res<ToolContext>,
    session: Res<crate::tools::ToolSessionState>,
    selection: Res<SelectionState>,
    query: Query<(Entity, Option<&Movable>, Option<&Deletable>), With<Interactive>>,
) {
    for (entity, movable, deletable) in &query {
        let mut highlight_kind = None;
        let is_selected = selection.primary == Some(entity);

        match *mode.get() {
            ToolMode::Move => {
                if let Some(crate::tools::ActiveToolSession::Move(move_session)) = &session.active {
                    if move_session.source_entity == entity {
                        // source of current move is not highlighted by normal hover/selection logic
                    }
                } else if tool.hovered_entity == Some(entity) {
                    if movable.is_some() {
                        highlight_kind = Some(HighlightKind::Hover);
                    }
                } else if is_selected {
                    highlight_kind = Some(HighlightKind::Selected);
                }
            }
            ToolMode::Delete => {
                if tool.hovered_entity == Some(entity) && deletable.is_some() {
                    highlight_kind = Some(HighlightKind::DeleteDanger);
                }
            }
            _ => {
                if tool.hovered_entity == Some(entity) {
                    highlight_kind = Some(HighlightKind::Hover);
                } else if is_selected {
                    highlight_kind = Some(HighlightKind::Selected);
                }
            }
        }

        if let Some(kind) = highlight_kind {
            commands.entity(entity).insert(HighlightIntent { kind });
        } else {
            commands.entity(entity).remove::<HighlightIntent>();
        }
    }
}

pub fn update_highlight_visuals(
    mut query: Query<
        (
            Option<&HighlightIntent>,
            Option<&PlacementPreview>,
            Option<&PreviewSource>,
            &mut Sprite,
        ),
        Without<crate::presentation::FootprintOutlineSegment>,
    >,
    _tool: Res<ToolContext>,
) {
    for (highlight, placement_preview, preview_source, mut sprite) in &mut query {
        let base_color = match highlight.map(|intent| intent.kind) {
            Some(HighlightKind::MoveInvalid | HighlightKind::BuildInvalid) => {
                Color::srgba(1.0, 0.32, 0.28, 0.82)
            }
            Some(HighlightKind::DeleteDanger) => Color::srgba(1.0, 0.20, 0.18, 0.90),
            Some(HighlightKind::MoveValid | HighlightKind::BuildValid) => {
                Color::srgba(0.45, 1.0, 0.55, 0.72)
            }
            Some(HighlightKind::Hover) => Color::srgb(1.0, 0.94, 0.62),
            Some(HighlightKind::Selected) => Color::srgb(0.62, 0.82, 1.0),
            None => Color::WHITE,
        };

        let final_color = if let Some(preview) = placement_preview {
            match &preview.validation {
                Some(Ok(())) => Color::srgba(0.45, 1.0, 0.55, 0.72),
                Some(Err(_)) => Color::srgba(1.0, 0.32, 0.28, 0.82),
                None => base_color.with_alpha(0.5),
            }
        } else if preview_source.is_some() {
            // Dim source object during Move
            base_color.with_alpha(0.3)
        } else {
            base_color
        };

        sprite.color = final_color;
    }
}
