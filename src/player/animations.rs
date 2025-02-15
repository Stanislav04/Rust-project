use super::Player;
use crate::GameState;
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_player_system,
                idle_animation_trigger_system,
                run_animation_trigger_system,
                jump_animation_trigger_system,
                fall_animation_trigger_system,
            )
                .run_if(in_state(GameState::MapExploring)),
        );
    }
}

#[derive(Default, Eq, PartialEq, Hash)]
pub enum AnimationType {
    #[default]
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Default, Component)]
pub struct AnimationInfo {
    animations: HashMap<AnimationType, Vec<usize>>,
    current_animation_type: AnimationType,
    current_animation: Vec<usize>,
    index: usize,
    timer: Timer,
}

impl AnimationInfo {
    pub fn new(
        animations: HashMap<AnimationType, Vec<usize>>,
        animation_type: AnimationType,
        timer: Timer,
    ) -> Self {
        let current_animation = animations
            .get(&animation_type)
            .expect("Animation type should have value in the map!")
            .clone();
        Self {
            animations,
            current_animation_type: animation_type,
            current_animation,
            index: 0,
            timer,
        }
    }

    fn set_animation(&mut self, animation_type: AnimationType) {
        if animation_type == self.current_animation_type {
            return;
        }
        let Some(animation) = self.animations.get(&animation_type) else {
            return;
        };
        self.current_animation_type = animation_type;
        self.current_animation = animation.clone();
        self.index = 0;
        self.timer.set_elapsed(self.timer.duration());
    }
}

fn idle_animation_trigger_system(
    rapier_context: Query<&RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if [AnimationType::Idle, AnimationType::Jump].contains(&animation_info.current_animation_type) {
        return;
    }
    if velocity.linvel.x != 0.0 {
        return;
    }
    for contact_pair in rapier_context.single().contact_pairs_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y == 0.0 {
                continue;
            }
            animation_info.set_animation(AnimationType::Idle);
            return;
        }
    }
}

fn run_animation_trigger_system(
    rapier_context: Query<&RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::Jump {
        return;
    }
    if velocity.linvel.x == 0.0 {
        return;
    }
    for contact_pair in rapier_context.single().contact_pairs_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y == 0.0 {
                continue;
            }
            animation_info.set_animation(AnimationType::Run);
            return;
        }
    }
}

fn jump_animation_trigger_system(
    mut events: EventReader<CollisionEvent>,
    mut animation_info: Query<(&mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::Jump {
        return;
    }
    println!("Before loop");
    for event in events.read() {
        println!("In loop");
        let CollisionEvent::Stopped(_, _, flags) = event else {
            continue;
        };
        if !flags.is_empty() {
            continue;
        }
        if velocity.linvel.y <= 0.0 {
            continue;
        }
        animation_info.set_animation(AnimationType::Jump);
        return;
    }
}

fn fall_animation_trigger_system(
    rapier_context: Query<&RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::Fall {
        return;
    }
    if velocity.linvel.y > 0.0 {
        return;
    }
    for contact_pair in rapier_context.single().contact_pairs_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y != 0.0 {
                return;
            }
        }
    }
    animation_info.set_animation(AnimationType::Fall);
}

fn animate_player_system(
    time: Res<Time>,
    mut animation_info: Query<(&mut Sprite, &mut AnimationInfo), With<Player>>,
) {
    let (mut sprite, mut animation_info) = animation_info.single_mut();
    if !animation_info.timer.tick(time.delta()).just_finished() {
        return;
    }
    animation_info.index = (animation_info.index + 1) % animation_info.current_animation.len();
    let Some(sprite) = &mut sprite.texture_atlas else {
        return;
    };
    sprite.index = animation_info.current_animation[animation_info.index];
}
