#[cfg(test)]
mod tests;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::objects::components::*;
use crate::tools::{
    ActiveToolSession, PlacementPreview, PreviewSource, SelectionState, ToolContext, ToolMode,
};

#[derive(Resource, Default)]
pub(crate) struct HighlightRuntimeState {
    hovered: Option<Entity>,
    selected: Option<Entity>,
    move_source: Option<Entity>,
    preview: Option<Entity>,
    dirty: Vec<Entity>,
}

fn push_unique(list: &mut Vec<Entity>, entity: Option<Entity>) {
    let Some(entity) = entity else {
        return;
    };
    if !list.contains(&entity) {
        list.push(entity);
    }
}

fn collect_dirty_entities(
    previous: &HighlightRuntimeState,
    current_hovered: Option<Entity>,
    current_selected: Option<Entity>,
    current_move_source: Option<Entity>,
    current_preview: Option<Entity>,
) -> Vec<Entity> {
    let mut dirty = Vec::with_capacity(8);
    push_unique(&mut dirty, previous.hovered);
    push_unique(&mut dirty, previous.selected);
    push_unique(&mut dirty, previous.move_source);
    push_unique(&mut dirty, previous.preview);
    push_unique(&mut dirty, current_hovered);
    push_unique(&mut dirty, current_selected);
    push_unique(&mut dirty, current_move_source);
    push_unique(&mut dirty, current_preview);
    dirty
}

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub(crate) struct HighlightIntentParams<'w, 's> {
    commands: Commands<'w, 's>,
    mode: Res<'w, State<ToolMode>>,
    tool: Res<'w, ToolContext>,
    session: Res<'w, crate::tools::ToolSessionState>,
    selection: Res<'w, SelectionState>,
    interactive: Query<
        'w,
        's,
        (
            Option<&'static Movable>,
            Option<&'static crate::objects::components::WallMovable>,
            Option<&'static Deletable>,
        ),
        With<Interactive>,
    >,
    state: ResMut<'w, HighlightRuntimeState>,
}

pub fn update_highlight_intents(mut params: HighlightIntentParams) {
    let current_mode = *params.mode.get();
    let current_hovered = params.tool.hovered_entity;
    let current_selected = params.selection.primary;
    let current_move_source = match &params.session.active {
        Some(ActiveToolSession::Move(session)) => match session {
            crate::tools::MoveToolSession::Floor(s) => Some(s.source_entity),
            crate::tools::MoveToolSession::WallMounted(s) => Some(s.source_entity),
            crate::tools::MoveToolSession::Door(s) => Some(s.source_entity),
        },
        _ => None,
    };
    let current_preview = match &params.session.active {
        Some(ActiveToolSession::Build(session)) => Some(session.preview_entity()),
        Some(ActiveToolSession::Move(session)) => match session {
            crate::tools::MoveToolSession::Floor(s) => Some(s.preview_entity),
            crate::tools::MoveToolSession::WallMounted(s) => Some(s.preview_entity),
            crate::tools::MoveToolSession::Door(s) => Some(s.preview_entity),
        },
        Some(ActiveToolSession::Expansion(_)) | None => None,
    };

    let dirty = collect_dirty_entities(
        &params.state,
        current_hovered,
        current_selected,
        current_move_source,
        current_preview,
    );

    for entity in &dirty {
        if let Ok(mut entity_commands) = params.commands.get_entity(*entity) {
            entity_commands.remove::<HighlightIntent>();
        }
    }

    if let Some(selected) = current_selected.filter(|entity| Some(*entity) != current_move_source)
        && let Ok(mut entity_commands) = params.commands.get_entity(selected)
    {
        entity_commands.insert(HighlightIntent {
            kind: HighlightKind::Selected,
        });
    }

    if let Some(hovered) = current_hovered
        .filter(|entity| Some(*entity) != current_selected && Some(*entity) != current_move_source)
    {
        let kind =
            match current_mode {
                ToolMode::Move => {
                    if params.interactive.get(hovered).ok().is_some_and(
                        |(movable, wall_movable, _)| movable.is_some() || wall_movable.is_some(),
                    ) {
                        Some(HighlightKind::Hover)
                    } else {
                        None
                    }
                }
                ToolMode::Delete => {
                    if params
                        .interactive
                        .get(hovered)
                        .ok()
                        .and_then(|(_, _, deletable)| deletable)
                        .is_some()
                    {
                        Some(HighlightKind::DeleteDanger)
                    } else {
                        None
                    }
                }
                _ => Some(HighlightKind::Hover),
            };

        if let Some(kind) = kind
            && let Ok(mut entity_commands) = params.commands.get_entity(hovered)
        {
            entity_commands.insert(HighlightIntent { kind });
        }
    }

    params.state.hovered = current_hovered;
    params.state.selected = current_selected;
    params.state.move_source = current_move_source;
    params.state.preview = current_preview;
    params.state.dirty = dirty;
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
    state: ResMut<'w, HighlightRuntimeState>,
}

fn base_color_from_intent(intent: Option<HighlightKind>) -> Color {
    match intent {
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
    }
}

fn final_sprite_color(
    highlight: Option<&HighlightIntent>,
    placement_preview: Option<&PlacementPreview>,
    preview_source: Option<&PreviewSource>,
) -> Color {
    let base_color = base_color_from_intent(highlight.map(|intent| intent.kind));

    if let Some(preview) = placement_preview {
        match &preview.validation {
            Some(Ok(())) => Color::srgba(0.45, 1.0, 0.55, 0.72),
            Some(Err(_)) => Color::srgba(1.0, 0.32, 0.28, 0.82),
            None => base_color.with_alpha(0.5),
        }
    } else if preview_source.is_some() {
        base_color.with_alpha(0.3)
    } else {
        base_color
    }
}

pub fn update_highlight_visuals(mut params: HighlightVisualParams) {
    let dirty = std::mem::take(&mut params.state.dirty);
    for entity in dirty {
        if let Ok((highlight, placement_preview, preview_source, mut sprite)) =
            params.query.get_mut(entity)
        {
            sprite.color = final_sprite_color(highlight, placement_preview, preview_source);
        }
    }
}
