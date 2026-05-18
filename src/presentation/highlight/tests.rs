use super::*;

#[test]
fn collect_dirty_entities_deduplicates_and_preserves_order() {
    let previous = HighlightRuntimeState {
        hovered: Some(Entity::from_bits(1)),
        selected: Some(Entity::from_bits(2)),
        move_source: Some(Entity::from_bits(3)),
        preview: Some(Entity::from_bits(4)),
        dirty: Vec::new(),
    };

    let dirty = collect_dirty_entities(
        &previous,
        Some(Entity::from_bits(3)),
        Some(Entity::from_bits(5)),
        Some(Entity::from_bits(4)),
        Some(Entity::from_bits(6)),
    );

    assert_eq!(
        dirty,
        vec![
            Entity::from_bits(1),
            Entity::from_bits(2),
            Entity::from_bits(3),
            Entity::from_bits(4),
            Entity::from_bits(5),
            Entity::from_bits(6),
        ]
    );
}

#[test]
fn final_sprite_color_respects_preview_and_source_states() {
    let hover = HighlightIntent {
        kind: HighlightKind::Hover,
    };

    assert_eq!(
        final_sprite_color(Some(&hover), None, None),
        Color::srgb(1.0, 0.94, 0.62)
    );

    let preview = PlacementPreview {
        validation: Some(Ok(())),
    };
    assert_eq!(
        final_sprite_color(Some(&hover), Some(&preview), None),
        Color::srgba(0.45, 1.0, 0.55, 0.72)
    );

    let invalid_preview = PlacementPreview {
        validation: Some(Err(
            crate::store::PlacementInvalidReason::OutsideWorldBounds,
        )),
    };
    assert_eq!(
        final_sprite_color(Some(&hover), Some(&invalid_preview), None),
        Color::srgba(1.0, 0.32, 0.28, 0.82)
    );

    assert_eq!(
        final_sprite_color(
            Some(&hover),
            None,
            Some(&PreviewSource {
                preview_entity: Entity::from_bits(9)
            })
        ),
        Color::srgb(1.0, 0.94, 0.62).with_alpha(0.3)
    );
}
