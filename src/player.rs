use crate::{map::Ground, GameState};
use animations::{AnimationInfo, AnimationType, AnimationsPlugin};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod animations;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationsPlugin)
            .add_systems(
                Update,
                player_movement_system.run_if(in_state(GameState::MapExploring)),
            )
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}

const PLAYER_WIDTH: f32 = 60.0;
const PLAYER_HEIGHT: f32 = 110.0;
const JUMP_POWER: f32 = 250.0;
const RUN_POWER: f32 = 100.0;

#[derive(Default, Component)]
pub struct Player;

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet("player/player_tilesheet.png", 80, 110, 9, 3, 0, 0, 24)]
    #[bundle()]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
    velocity: Velocity,
    player: Player,
}

#[derive(Default, Bundle)]
struct ColliderBundle {
    collider: Collider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    friction: Friction,
    active_events: ActiveEvents,
    animation_info: AnimationInfo,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(_: &EntityInstance) -> Self {
        Self {
            collider: Collider::cuboid(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            active_events: ActiveEvents::all(),
            animation_info: AnimationInfo::new(
                HashMap::from_iter([
                    (AnimationType::Idle, vec![0]),
                    (AnimationType::Run, vec![9, 10]),
                    (AnimationType::Jump, vec![1]),
                    (AnimationType::Fall, vec![2]),
                ]),
                AnimationType::Idle,
                Timer::from_seconds(0.2, TimerMode::Repeating),
            ),
        }
    }
}

fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    rapier_context: Query<&RapierContext>,
    mut player_info: Query<(Entity, &mut Velocity, &mut Sprite), With<Player>>,
    tile_info: Query<Entity, With<Ground>>,
) {
    let (player, mut velocity, mut sprite) = player_info.single_mut();
    let up = keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]);
    let left = keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);

    velocity.linvel.x = if left {
        sprite.flip_x = true;
        -RUN_POWER
    } else if right {
        sprite.flip_x = false;
        RUN_POWER
    } else {
        0.0
    };

    if up {
        for tile in tile_info.iter() {
            let Some(contact_pair) = rapier_context.single().contact_pair(player, tile) else {
                continue;
            };
            for manifold in contact_pair.manifolds() {
                let first_entity = manifold
                    .rigid_body1()
                    .expect("An entity is expected when collision is detected!");
                if (first_entity == player && manifold.normal().y == -1.0)
                    || manifold.normal().y == 1.0
                {
                    velocity.linvel.y = JUMP_POWER;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_changes_velocity_horizontally() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((Player, Velocity::default(), Sprite::default()))
            .id();
        app.world_mut().spawn(RapierContext::default());
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, player_movement_system);

        // When
        input.press(KeyCode::ArrowRight);
        app.insert_resource(input);
        app.update();

        // Then
        let velocity = app.world().get::<Velocity>(player).unwrap();
        assert_eq!(velocity.linvel.x, RUN_POWER);
        assert_eq!(velocity.linvel.y, 0.0);
    }

    #[test]
    fn test_not_running_nullifies_velocity() {
        // Given
        let mut app = App::new();

        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::linear(Vec2::new(RUN_POWER, 0.0)),
                Sprite::default(),
            ))
            .id();
        app.world_mut().spawn(RapierContext::default());
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, player_movement_system);

        // When
        app.update();

        // Then
        let velocity = app.world().get::<Velocity>(player).unwrap();
        assert_eq!(velocity.linvel.x, 0.0);
        assert_eq!(velocity.linvel.y, 0.0);
    }

    #[test]
    fn test_running_left_flips_sprite_correctly() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite {
                    flip_x: false,
                    ..default()
                },
            ))
            .id();
        app.world_mut().spawn(RapierContext::default());
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, player_movement_system);

        // When
        input.press(KeyCode::ArrowLeft);
        app.insert_resource(input);
        app.update();

        // Then
        let sprite = app.world().get::<Sprite>(player).unwrap();
        assert!(sprite.flip_x);
    }

    #[test]
    fn test_running_right_flips_sprite_correctly() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite {
                    flip_x: true,
                    ..default()
                },
            ))
            .id();
        app.world_mut().spawn(RapierContext::default());
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, player_movement_system);

        // When
        input.press(KeyCode::ArrowRight);
        app.insert_resource(input);
        app.update();

        // Then
        let sprite = app.world().get::<Sprite>(player).unwrap();
        assert!(!sprite.flip_x);
    }

    #[test]
    fn test_not_running_does_not_change_sprite_flip() {
        // Given
        let mut app = App::new();

        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite {
                    flip_x: true,
                    ..default()
                },
            ))
            .id();
        app.world_mut().spawn(RapierContext::default());
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, player_movement_system);

        // When
        app.update();

        // Then
        let sprite = app.world().get::<Sprite>(player).unwrap();
        assert!(sprite.flip_x);
    }

    #[test]
    fn test_jumping_on_ground_changes_velocity_vertically() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite::default(),
                Collider::cuboid(10.0, 10.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                RigidBody::Dynamic,
            ))
            .id();
        app.world_mut().spawn((
            Ground,
            Collider::cuboid(10.0, 10.0),
            Transform::from_xyz(0.0, -20.0, 0.0),
            RigidBody::Fixed,
        ));
        app.init_resource::<Time>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_systems(Update, player_movement_system);

        // When
        app.update();
        input.press(KeyCode::ArrowUp);
        app.insert_resource(input);
        app.update();

        // Then
        let velocity = app.world().get::<Velocity>(player).unwrap();
        assert_eq!(velocity.linvel.x, 0.0);
        assert_eq!(velocity.linvel.y, JUMP_POWER);
    }

    #[test]
    fn test_jumping_on_nonground_does_not_change_velocity_vertically() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite::default(),
                Collider::cuboid(10.0, 10.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                RigidBody::Dynamic,
            ))
            .id();
        app.world_mut().spawn((
            Collider::cuboid(10.0, 10.0),
            Transform::from_xyz(0.0, -20.0, 0.0),
            RigidBody::Fixed,
        ));
        app.init_resource::<Time>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_systems(Update, player_movement_system);

        // When
        app.update();
        input.press(KeyCode::ArrowUp);
        app.insert_resource(input);
        app.update();

        // Then
        let velocity = app.world().get::<Velocity>(player).unwrap();
        assert_eq!(velocity.linvel.x, 0.0);
        assert_eq!(velocity.linvel.y, 0.0);
    }

    #[test]
    fn test_jumping_from_the_side_of_ground_does_not_change_velocity_vertically() {
        // Given
        let mut app = App::new();

        let mut input = ButtonInput::<KeyCode>::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Velocity::default(),
                Sprite::default(),
                Collider::cuboid(10.0, 10.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                RigidBody::Dynamic,
            ))
            .id();
        app.world_mut().spawn((
            Ground,
            Collider::cuboid(10.0, 10.0),
            Transform::from_xyz(-20.0, 0.0, 0.0),
            RigidBody::Fixed,
        ));
        app.init_resource::<Time>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_systems(Update, player_movement_system);

        // When
        app.update();
        input.press(KeyCode::ArrowUp);
        app.insert_resource(input);
        app.update();

        // Then
        let velocity = app.world().get::<Velocity>(player).unwrap();
        assert_eq!(velocity.linvel.x, 0.0);
        assert_eq!(velocity.linvel.y, 0.0);
    }
}
