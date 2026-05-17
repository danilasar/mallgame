use bevy::prelude::*;

use crate::ui::BlocksWorldInput;

pub fn ui_button(label: &'static str, width: f32, height: f32) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.18, 0.16, 0.14)),
        BlocksWorldInput,
        Name::new(label),
    )
}

pub fn label_text(label: &'static str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.84, 0.45)),
    )
}
