use crate::player::Player;
use crate::GameState;
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod nodes;
pub struct RiddlesPlugin;

impl Plugin for RiddlesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnsweredRiddles::default())
            .add_systems(OnExit(GameState::LevelLoading), init_riddles_system)
            .add_systems(
                Update,
                touch_door_system.run_if(in_state(GameState::MapExploring)),
            )
            .add_systems(
                Update,
                (
                    answering_riddle_system,
                    delete_digit_system,
                    (correct_answer_system, clear_input_system).chain(),
                    close_riddle_system,
                )
                    .run_if(in_state(GameState::RiddleSolving)),
            );
    }
}

#[derive(Default, Resource)]
struct AnsweredRiddles {
    ids: HashSet<String>,
}

#[derive(Component)]
struct RiddleNode;

#[derive(Component)]
struct AnswerContainer {
    index: usize,
    answer_length: usize,
}

#[derive(Component)]
struct Answer {
    position: usize,
}

#[derive(Default, Component)]
pub struct RiddleInfo {
    question: String,
    answer: String,
    riddle: Option<Entity>,
    next_level: String,
}

impl From<&EntityInstance> for RiddleInfo {
    fn from(entity_instance: &EntityInstance) -> Self {
        let fields = HashMap::from_iter(entity_instance.field_instances.iter().map(|field| {
            (
                field.identifier.clone(),
                match &field.value {
                    FieldValue::String(Some(value)) => value.clone(),
                    _ => "".to_string(),
                },
            )
        }));
        Self {
            question: fields
                .get("question")
                .expect("A question is required for a riddle!")
                .into(),
            answer: fields
                .get("answer")
                .expect("An answer is required for a riddle!")
                .into(),
            next_level: fields
                .get("next_level")
                .expect("A next level is required for a riddle!")
                .into(),
            ..default()
        }
    }
}

fn init_riddles_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    answered_riddles: Res<AnsweredRiddles>,
    mut doors: Query<(&mut RiddleInfo, &mut Sprite)>,
) {
    use nodes::*;

    for (mut door, mut sprite) in doors.iter_mut() {
        if answered_riddles.ids.contains(&door.question) {
            if let Some(sprite) = &mut sprite.texture_atlas {
                sprite.index = 75;
            }
            continue;
        }
        door.riddle = Some(
            commands
                .spawn(root_node())
                .insert(RiddleNode)
                .with_children(|parent| {
                    parent.spawn(question_text(&asset_server, &door.question.clone()));
                    parent
                        .spawn(answer_container())
                        .insert(AnswerContainer {
                            index: 0,
                            answer_length: door.answer.len(),
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(answer_position(&asset_server))
                                .insert(TextColor(Color::srgb(1.0, 0.0, 0.0)))
                                .insert(Answer { position: 0 });
                            parent
                                .spawn(answer_position(&asset_server))
                                .insert(TextColor(Color::srgb(0.0, 0.0, 1.0)))
                                .insert(Answer { position: 1 });
                            parent
                                .spawn(answer_position(&asset_server))
                                .insert(TextColor(Color::srgb(1.0, 1.0, 0.0)))
                                .insert(Answer { position: 2 });
                        });
                })
                .id(),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn touch_door_system(
    answered_riddles: Res<AnsweredRiddles>,
    mut next_state: ResMut<NextState<GameState>>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut current_level: ResMut<LevelSelection>,
    rapier_context: Query<&RapierContext>,
    player_info: Query<Entity, With<Player>>,
    mut doors: Query<(Entity, &mut RiddleInfo)>,
    mut riddle_nodes: Query<&mut Visibility, With<RiddleNode>>,
) {
    let player = player_info.single();
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }
    for (door, riddle_info) in doors.iter_mut() {
        if rapier_context.single().intersection_pair(player, door) != Some(true) {
            continue;
        };
        if answered_riddles.ids.contains(&riddle_info.question) {
            keyboard_input.reset(KeyCode::Space);
            *current_level = LevelSelection::iid(riddle_info.next_level.clone());
            next_state.set(GameState::LevelLoading);
            return;
        }
        let mut node_visibility = riddle_nodes
            .get_mut(
                riddle_info
                    .riddle
                    .expect("The riddle entity is supposed to be set by the init_riddles_system!"),
            )
            .unwrap();
        *node_visibility = Visibility::Visible;
        next_state.set(GameState::RiddleSolving);
        return;
    }
}

fn answering_riddle_system(
    mut input: EventReader<KeyboardInput>,
    mut container_info: Query<(&mut AnswerContainer, &InheritedVisibility)>,
    mut answer_nodes: Query<(&mut Text, &InheritedVisibility, &Answer)>,
) {
    for character in input.read() {
        if !(character.state.is_pressed()
            && (KeyCode::Digit0..=KeyCode::Digit9).contains(&character.key_code))
        {
            continue;
        }
        let (mut container, _) = container_info
            .iter_mut()
            .find(|(_, visibility)| visibility.get())
            .expect("A visible container is expected while this system is running!");
        let (mut answer, _, _) = answer_nodes
            .iter_mut()
            .filter(|(_, visibility, _)| visibility.get())
            .find(|(_, _, answer)| answer.position == container.index)
            .expect("The container is expected to have answer positions and the container's index is always valid!");
        if let Key::Character(character) = &character.logical_key {
            answer.0 = character.to_string();
        }
        container.index = (container.index + 1) % container.answer_length;
    }
}

fn delete_digit_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut container_info: Query<(&mut AnswerContainer, &InheritedVisibility)>,
    mut answer_nodes: Query<(&mut Text, &InheritedVisibility, &Answer)>,
) {
    if !keyboard_input.just_pressed(KeyCode::Backspace) {
        return;
    }
    let (mut container, _) = container_info
        .iter_mut()
        .find(|(_, visibility)| visibility.get())
        .expect("A visible container is expected while this system is running!");
    if container.index == 0 {
        container.index = container.answer_length;
    }
    container.index -= 1;
    let (mut answer, _, _) = answer_nodes
            .iter_mut()
            .find(|(_, visibility, answer)| visibility.get() && answer.position == container.index)
            .expect("The container is expected to have answer positions and the container's index is always valid!");
    answer.0 = "_".to_string();
}

fn correct_answer_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut answered_riddles: ResMut<AnsweredRiddles>,
    mut next_state: ResMut<NextState<GameState>>,
    mut doors: Query<(&mut RiddleInfo, &mut Sprite, &InheritedVisibility)>,
    answer_nodes: Query<(&Text, &InheritedVisibility, &Answer)>,
) {
    if !keyboard_input.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        return;
    }
    let mut answer_nodes = Vec::from_iter(
        answer_nodes
            .iter()
            .filter(|(_, visibility, _)| visibility.get())
            .map(|(text, _, pos)| (pos.position, text.0.clone())),
    );
    answer_nodes.sort_by_key(|(pos, _)| *pos);
    let answer = answer_nodes
        .into_iter()
        .map(|(_, value)| value)
        .collect::<String>();
    let (door, mut sprite, _) = doors
        .iter_mut()
        .find(|(_, _, visibility)| visibility.get())
        .expect("Only one door should be active while answering a riddle!");
    if answer != door.answer {
        return;
    }
    answered_riddles.ids.insert(door.question.clone());
    commands
        .entity(
            door.riddle
                .expect("The riddle entity is supposed to be set by the init_riddles_system!"),
        )
        .despawn_recursive();
    if let Some(sprite) = &mut sprite.texture_atlas {
        sprite.index = 75;
    }
    next_state.set(GameState::MapExploring);
}

fn clear_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut container_info: Query<(&mut AnswerContainer, &InheritedVisibility)>,
    mut answer_nodes: Query<(&mut Text, &InheritedVisibility), With<Answer>>,
) {
    if !keyboard_input.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        return;
    }
    if answer_nodes
        .iter_mut()
        .filter(|(_, visibility)| visibility.get())
        .any(|(text, _)| text.0 == *"_")
    {
        return;
    }
    if let Some((mut container, _)) = container_info
        .iter_mut()
        .find(|(_, visibility)| visibility.get())
    {
        container.index = 0;
    }
    answer_nodes
        .iter_mut()
        .filter(|(_, visibility)| visibility.get())
        .for_each(|(mut text, _)| {
            text.0 = "_".to_string();
        });
}

fn close_riddle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut riddle_nodes: Query<&mut Visibility, With<RiddleNode>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }
    let mut riddle_visibility = riddle_nodes
        .iter_mut()
        .find(|visibility| **visibility == Visibility::Visible)
        .expect("Exactly one visible riddle node is expected while this system is running!");
    *riddle_visibility = Visibility::Hidden;
    next_state.set(GameState::MapExploring);
}
