use avian2d::prelude::{
    ExternalForce, ExternalImpulse, LinearVelocity, Physics, PhysicsTime, Rotation,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_kira_audio::{Audio, AudioControl};

use crate::player::abilities::{go_into_bullet_time, reset_bullet_time};
use crate::player::movement::AuraEarned;
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
pub mod abilities;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
        spawn::plugin,
        assets::plugin,
        engine::plugin,
        hud::plugin,
        death::plugin,
        // dash::plugin,
        sound::plugin,
    ))
    .add_systems(
        Update,
        (
            camera_follow_player.run_if(in_state(IntroState(false))),
            go_into_bullet_time.run_if(input_just_pressed(KeyCode::Space)),
            reset_bullet_time,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.insert_resource(Score(0.0));

    app.add_systems(OnEnter(Screen::Gameplay), |mut score: ResMut<Score>| {
        score.0 = 0.0
    });
}

#[derive(Resource)]
pub struct Score(pub f32);

#[derive(Component, Default)]
pub struct Player {
    pub aura_points: f32, // given based on style (flying by objects at high speeds, etc.) // * maybe change this to a float too
    pub bullet_time_until: f32, // seconds
    pub bullet_time_cooldown_until: f32, // seconds
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
}
