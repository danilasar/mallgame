use bevy::prelude::*;

use crate::ui::BlocksWorldInput;

#[derive(Resource, Debug, Clone)]
pub struct UiFonts {
    pub regular: Handle<Font>,
}

pub fn load_ui_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFonts {
        regular: asset_server.load("fonts/IosevkaNerdFont-Regular.ttf"),
    });
}

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

pub fn label_text(label: impl Into<String>, fonts: &UiFonts) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font: fonts.regular.clone(),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.84, 0.45)),
    )
}

pub fn ui_text(
    label: impl Into<String>,
    font_size: f32,
    color: Color,
    fonts: &UiFonts,
) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font: fonts.regular.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
    )
}
