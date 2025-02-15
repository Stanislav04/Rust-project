use bevy::{prelude::*, window::CursorOptions};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use map::MapPlugin;
use player::PlayerPlugin;
use riddles::RiddlesPlugin;

mod map;
mod player;
mod riddles;

#[derive(Clone, Debug, Eq, PartialEq, Hash, States)]
enum GameState {
    MapExploring,
    RiddleSolving,
    LevelLoading,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Puzzle Up".to_string(),
                resizable: false,
                cursor_options: CursorOptions {
                    visible: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .insert_state(GameState::LevelLoading)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(LdtkPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(RiddlesPlugin)
        .add_systems(Startup, setup_system)
        .run();
}

fn setup_system(mut commands: Commands, mut rapier_config: Query<&mut RapierConfiguration>) {
    commands.spawn(Camera2d);
    rapier_config.single_mut().gravity = Vec2::new(0.0, -400.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_system_spawns_camera() {
        // Given
        let mut app = App::new();

        app.world_mut().spawn(RapierConfiguration::new(100.0));

        // When
        app.add_systems(Startup, setup_system);
        app.update();

        // Then
        let cameras = app.world_mut().query::<&Camera2d>().iter(app.world()).len();
        assert_eq!(cameras, 1);
    }

    #[test]
    fn test_setup_system_updates_gravity() {
        // Given
        let mut app = App::new();

        let rapier_configuration = app
            .world_mut()
            .spawn(RapierConfiguration {
                gravity: Vec2::new(0.0, 0.0),
                physics_pipeline_active: true,
                query_pipeline_active: true,
                scaled_shape_subdivision: 0,
                force_update_from_transform_changes: true,
            })
            .id();

        // When
        app.add_systems(Startup, setup_system);
        app.update();

        // Then
        let rapier_configuration = app
            .world()
            .get::<RapierConfiguration>(rapier_configuration)
            .unwrap();
        assert_eq!(rapier_configuration.gravity, Vec2::new(0.0, -400.0));
    }
}
