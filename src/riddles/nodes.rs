use bevy::prelude::*;

pub fn root_node() -> (Node, BackgroundColor, Visibility) {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.85)),
        Visibility::Hidden,
    )
}

pub fn question_text(
    asset_server: &Res<AssetServer>,
    question: &str,
) -> (Text, TextColor, TextFont, TextLayout) {
    (
        Text(question.to_string()),
        TextColor(Color::WHITE),
        TextFont {
            font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
            font_size: 60.0,
            ..default()
        },
        TextLayout {
            justify: JustifyText::Center,
            ..Default::default()
        },
    )
}

pub fn answer_container() -> (Node, BackgroundColor) {
    (
        Node {
            justify_content: JustifyContent::SpaceAround,
            width: Val::Percent(30.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

pub fn answer_position(asset_server: &Res<AssetServer>) -> (Text, TextFont) {
    (
        Text("_".to_string()),
        TextFont {
            font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
            font_size: 60.0,
            ..default()
        },
    )
}
