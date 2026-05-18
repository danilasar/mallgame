use bevy::prelude::*;

use crate::objects::components::*;
use crate::tools::{PlacementPreview, PreviewSource, SelectionState, ToolContext, ToolMode};
use bevy::ecs::system::SystemParam;

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct HighlightIntentParams<'w, 's> {
    commands: Commands<'w, 's>,
    mode: Res<'w, State<ToolMode>>,
    tool: Res<'w, ToolContext>,
    session: Res<'w, crate::tools::ToolSessionState>,
    selection: Res<'w, SelectionState>,
    query: Query<'w, 's, (Entity, Option<&'static Movable>, Option<&'static Deletable>), With<Interactive>>,
}

pub fn update_highlight_intents(mut params: HighlightIntentParams) {
    for (entity, movable, deletable) in &params.query {
        let mut highlight_kind = None;
        let is_selected = params.selection.primary == Some(entity);

        match *params.mode.get() {
            ToolMode::Move => {
                if let Some(crate::tools::ActiveToolSession::Move(move_session)) = &params.session.active {
                    if move_session.source_entity == entity {
                        // source of current move is not highlighted by normal hover/selection logic
                    }
                } else if params.tool.hovered_entity == Some(entity) {
                    if movable.is_some() {
                        highlight_kind = Some(HighlightKind::Hover);
                    }
                } else if is_selected {
                    highlight_kind = Some(HighlightKind::Selected);
                }
            }
            ToolMode::Delete => {
                if params.tool.hovered_entity == Some(entity) && deletable.is_some() {
                    highlight_kind = Some(HighlightKind::DeleteDanger);
                }
            }
            _ => {
                if params.tool.hovered_entity == Some(entity) {
                    highlight_kind = Some(HighlightKind::Hover);
                } else if is_selected {
                    highlight_kind = Some(HighlightKind::Selected);
                }
            }
        }

        if let Some(kind) = highlight_kind {
            params.commands.entity(entity).insert(HighlightIntent { kind });
        } else {
            params.commands.entity(entity).remove::<HighlightIntent>();
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct HighlightVisualParams<'w, 's> {
    query: Query<
        'w,
        's,
        (
            Option<&'static HighlightIntent>,
            Option<&'static PlacementPreview>,
            Option<&'static PreviewSource>,
            &'static mut Sprite,
        ),
        Without<crate::presentation::FootprintOutlineSegment>,
    >,
    _tool: Res<'w, ToolContext>,
}

pub fn update_highlight_visuals(mut params: HighlightVisualParams) {
    for (highlight, placement_preview, preview_source, mut sprite) in &mut params.query {
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
