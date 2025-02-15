use crate::{riddles::RiddleInfo, GameState};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use text::*;

mod text;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::iid(STARTING_LEVEL))
            .add_systems(Startup, zone_text_setup_system)
            .add_systems(OnEnter(GameState::LevelLoading), level_loading_system)
            .add_systems(
                OnExit(GameState::LevelLoading),
                (center_map_system, normalize_font_system),
            )
            .add_systems(Startup, map_setup_system)
            .add_systems(
                Update,
                (
                    level_loaded_system.run_if(in_state(GameState::LevelLoading)),
                    (show_zone_text_system, hide_zone_text_system)
                        .run_if(in_state(GameState::MapExploring)),
                ),
            )
            .register_ldtk_entity::<GroundTile>("Ground")
            .register_ldtk_entity::<GroundTile>("LevelBorder")
            .register_ldtk_entity::<Door>("Door")
            .register_ldtk_entity::<BoxTile>("Box")
            .register_ldtk_entity::<TextSignBundle>("TextSign")
            .register_ldtk_entity::<ZoneTextBundle>("ZoneText");
    }
}

const STARTING_LEVEL: &str = "27e654c0-ed50-11ed-9ee3-a3abea3fe6ae";
const SMALL_TILE_SIZE: f32 = 16.0;
const LARGE_TILE_SIZE: f32 = 64.0;

fn map_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk").into(),
        ..default()
    });
}

fn level_loading_system(
    level_selection: Res<LevelSelection>,
    mut level_set_info: Query<&mut LevelSet>,
) {
    let Some(mut level_set) = level_set_info.iter_mut().next() else {
        return;
    };
    level_set.iids.clear();
    let LevelSelection::Identifier(level_id) = &*level_selection else {
        return;
    };
    level_set.iids.insert(LevelIid::from(level_id.clone()));
}

fn level_loaded_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut events: EventReader<LevelEvent>,
) {
    for event in events.read() {
        let LevelEvent::Spawned(_) = event else {
            continue;
        };
        next_state.set(GameState::MapExploring);
    }
}

fn center_map_system(
    current_level: Res<LevelSelection>,
    // levels: Query<(&LevelIid, &GlobalTransform)>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    mut map_info: Query<&mut Transform, With<LevelIid>>,
) {
    let LevelSelection::Iid(ref level_id) = *current_level else {
        return;
    };
    let Some(level) = ldtk_project_assets
        .get(ldtk_projects.single())
        .expect("ldtk project should be loaded before player is spawned")
        .get_raw_level_by_iid(level_id.get())
    else {
        return;
    };
    let mut map = map_info.single_mut();
    // let level_ = levels.get(handle).unwrap();
    map.translation.x = -level.px_wid as f32 / 2.0;
    map.translation.y = -level.px_hei as f32 / 2.0;
}

#[derive(Default, Component)]
pub struct Ground;

#[derive(Default, Bundle, LdtkEntity)]
struct GroundTile {
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
    ground: Ground,
}

#[derive(Default, Bundle, LdtkEntity)]
struct LevelBorder {
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
}

#[derive(Default, Bundle, LdtkEntity)]
struct BoxTile {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
    ground: Ground,
}

#[derive(Default, Bundle, LdtkEntity)]
struct Door {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    #[bundle()]
    collider_bundle: ColliderBundle,
    sensor: Sensor,
    #[from_entity_instance]
    riddle_info: RiddleInfo,
}

#[derive(Default, Bundle)]
struct ColliderBundle {
    collider: Collider,
    rigid_body: RigidBody,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "Ground" => Self {
                collider: Collider::cuboid(SMALL_TILE_SIZE / 2.0, SMALL_TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            "LevelBorder" => Self {
                collider: Collider::cuboid(SMALL_TILE_SIZE / 2.0, SMALL_TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            "Door" | "Box" | "ZoneText" => Self {
                collider: Collider::cuboid(LARGE_TILE_SIZE / 2.0, LARGE_TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            _ => Self::default(),
        }
    }
}
