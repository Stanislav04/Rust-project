use super::ColliderBundle;
use bevy::{prelude::*, text::TextBounds, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Default, Component)]
pub struct StaticText;

#[derive(Default, LdtkEntity, Bundle)]
pub struct TextSignBundle {
    #[from_entity_instance]
    #[bundle()]
    text_sign: TextSign,
    static_text: StaticText,
}

#[derive(Default, Bundle)]
struct TextSign {
    text: Text2d,
    text_font: TextFont,
    text_bounds: TextBounds,
    text_color: TextColor,
    text_layout: TextLayout,
    transform: Transform,
}

impl From<&EntityInstance> for TextSign {
    fn from(entity_instance: &EntityInstance) -> Self {
        let fields = HashMap::from_iter(entity_instance.field_instances.iter().map(|field| {
            (
                field.identifier.clone(),
                match field.value.clone() {
                    FieldValue::String(Some(value)) => value,
                    FieldValue::Float(Some(value)) => value.to_string(),
                    _ => "".to_string(),
                },
            )
        }));
        Self {
            text: Text2d::new(
                fields
                    .get("text")
                    .expect("Text is expected for a text sign!"),
            ),
            text_bounds: TextBounds::new(
                entity_instance.width as f32,
                entity_instance.height as f32,
            ),
            text_color: TextColor(
                entity_instance
                    .field_instances
                    .iter()
                    .find(|field| field.identifier == "color")
                    .map(|field| {
                        if let FieldValue::Color(color) = field.value {
                            color
                        } else {
                            default()
                        }
                    })
                    .expect("Default color is expected to be set by the editor!"),
            ),
            text_font: TextFont {
                font_size: fields
                    .get("font_size")
                    .expect("The font size of the text is expected!")
                    .parse()
                    .expect("Font size is expected to be a number!"),
                ..default()
            },
            text_layout: TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            ..default()
        }
    }
}

pub fn normalize_font_system(
    asset_server: Res<AssetServer>,
    mut text_query: Query<(&mut TextFont, &mut Transform), With<StaticText>>,
) {
    for (mut text_font, mut transform) in text_query.iter_mut() {
        text_font.font = asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf");
        transform.scale = Vec3::new(1.0, 1.0, 1.0);
    }
}

#[derive(Default, Component)]
pub struct ZoneText;

#[derive(Default, Component)]
pub struct CommonTextContainer;

#[derive(Default, Component)]
pub struct CommonTextContent;

#[derive(Default, Bundle, LdtkEntity)]
pub struct ZoneTextBundle {
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
    sensor: Sensor,
    #[from_entity_instance]
    text_info: TextInfo,
    zone_text: ZoneText,
}

#[derive(Default, Component)]
pub struct TextInfo {
    text: String,
}

impl From<&EntityInstance> for TextInfo {
    fn from(entity_instance: &EntityInstance) -> Self {
        let fields = HashMap::from_iter(entity_instance.field_instances.iter().map(|field| {
            (
                field.identifier.clone(),
                match field.value.clone() {
                    FieldValue::String(Some(value)) => value,
                    _ => "".to_string(),
                },
            )
        }));
        Self {
            text: fields
                .get("text")
                .expect("Text to be displayed is expected!")
                .clone(),
        }
    }
}

fn zone_text_node() -> (Node, BackgroundColor, Visibility) {
    (
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(20.0),
            position_type: PositionType::Absolute,
            bottom: Val::Percent(0.0),
            padding: UiRect::all(Val::Percent(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.85)),
        Visibility::Hidden,
    )
}

fn zone_text_content(asset_server: &Res<AssetServer>) -> (Text, TextColor, TextFont) {
    (
        Text("".to_string()),
        TextColor(Color::WHITE),
        TextFont {
            font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
            font_size: 30.0,
            ..default()
        },
    )
}

pub fn zone_text_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(zone_text_node())
        .insert(CommonTextContainer)
        .with_children(|parent| {
            parent
                .spawn(zone_text_content(&asset_server))
                .insert(CommonTextContent);
        });
}

pub fn show_zone_text_system(
    mut events: EventReader<CollisionEvent>,
    mut text_container_info: Query<&mut Visibility, With<CommonTextContainer>>,
    mut text_content_info: Query<&mut Text, With<CommonTextContent>>,
    zone_texts: Query<&TextInfo, With<ZoneText>>,
) {
    for event in events.read() {
        let CollisionEvent::Started(entity, other, _) = event else {
            continue;
        };
        if !(zone_texts.contains(*entity) || zone_texts.contains(*other)) {
            continue;
        }

        let mut container_visibility = text_container_info.single_mut();
        let mut text_content = text_content_info.single_mut();
        let zone_text = zone_texts.get(*entity).unwrap_or_else(|_| {
            zone_texts
                .get(*other)
                .expect("One of the colliders is expected to be a zone text!")
        });
        text_content.0 = zone_text.text.clone();
        *container_visibility = Visibility::Visible;
    }
}

pub fn hide_zone_text_system(
    mut events: EventReader<CollisionEvent>,
    mut text_container_info: Query<&mut Visibility, With<CommonTextContainer>>,
    mut text_content_info: Query<&mut Text, With<CommonTextContent>>,
    zone_texts: Query<&TextInfo, With<ZoneText>>,
) {
    for event in events.read() {
        let CollisionEvent::Stopped(entity, other, _) = event else {
            continue;
        };
        if !(zone_texts.contains(*entity) || zone_texts.contains(*other)) {
            continue;
        }

        let mut container_visibility = text_container_info.single_mut();
        let mut text_content = text_content_info.single_mut();
        *container_visibility = Visibility::Hidden;
        text_content.0 = "".to_string();
    }
}
