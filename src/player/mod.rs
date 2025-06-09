use avian2d::prelude::{
    ExternalForce, ExternalImpulse, LinearVelocity, Physics, PhysicsTime, Rotation,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_kira_audio::{Audio, AudioControl};

use crate::space::intro::IntroState;
use crate::{player::movement::DashTimer, screens::Screen};

pub mod assets;
pub mod dash;
pub mod death;
pub mod engine;
pub mod hud;
pub mod movement;
pub mod sound;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
        spawn::plugin,
        assets::plugin,
        engine::plugin,
        hud::plugin,
        death::plugin,
        dash::plugin,
        sound::plugin,
    ))
    .add_systems(
        Update,
        (
            camera_follow_player.run_if(in_state(IntroState(false))),
            go_into_bullet_time.run_if(input_just_pressed(KeyCode::Space)),
            player_powers,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.insert_resource(Score(0.0));
}

#[derive(Resource)]
pub struct Score(pub f32);

#[derive(Component, Default)]
pub struct Player {
    pub aura_points: f32, // given based on style (flying by objects at high speeds, etc.) // * maybe change this to a float too
    pub bullet_time_until: f32, // seconds
    pub bullet_time_cooldown_until: f32, // seconds
    pub dash_timer: DashTimer,
    pub near_asteroids: bool,
}

// TODO: ensure it runs in the right schedule
pub fn camera_follow_player(
    q_camera: Single<&mut Transform, With<Camera>>,
    q_player: Single<(&GlobalTransform, &LinearVelocity), With<Player>>,
    time: Res<Time>,
) {
    let mut cam_transform = q_camera.into_inner();

    let (player_transform, vel) = q_player.into_inner();

    let vel_len = vel.0.length();

    let z = cam_transform
        .translation
        .z
        .lerp(230.0 + vel_len * 1.0, time.delta_secs());

    cam_transform.translation = player_transform.translation().with_z(z);
    // cam_transform.translation = cam_transform.translation.move_towards(player_transform.translation().with_z(30.0 + 1.0 * vel_len), 45.0 * time.delta_secs());

    // cam_transform.translation = player_transform.translation().with_z(50.0 + 10.0 * vel_len);

    // cam_transform.translation.y += -vel_len / 4.0;

    // cam_transform.rotation = Quat::from_rotation_x(vel_len / 70.0);
}

const BULLET_TIME_DURATION: f32 = 2.0;
const BULLET_TIME_COOLDOWN: f32 = 3.0; // seconds
const BULLET_TIME_AURA_COST: f32 = 100.0;

fn player_powers(
    player: Single<&Player>,
    real_time: Res<Time>,
    mut physics_time: ResMut<Time<Physics>>,
    audio: Res<Audio>,
) {
    if real_time.elapsed_secs() > player.bullet_time_until {
        physics_time.set_relative_speed(1.0);
        audio.set_playback_rate(1.0);
    }
}

fn go_into_bullet_time(
    real_time: Res<Time>,
    mut physics_time: ResMut<Time<Physics>>,
    mut player: Single<&mut Player>,
    audio: Res<Audio>,
) {
    // TODO: PLAY SOUND HERE

    let rt = real_time.elapsed_secs();

    if rt < player.bullet_time_until
        || rt < player.bullet_time_cooldown_until
        || player.aura_points < BULLET_TIME_AURA_COST
    {
        return;
    }
    physics_time.set_relative_speed(0.25);
    audio.set_playback_rate(0.25);
    player.bullet_time_until = rt + BULLET_TIME_DURATION;
    player.bullet_time_cooldown_until = rt + BULLET_TIME_DURATION + BULLET_TIME_COOLDOWN;
    player.aura_points -= BULLET_TIME_AURA_COST
}
