use avian2d::prelude::{
    ExternalForce, ExternalImpulse, LinearVelocity, Physics, PhysicsTime, Rotation,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_kira_audio::{Audio, AudioControl};

use crate::player::movement::AuraEarned;
use crate::player::Player;
use crate::space::intro::IntroState;
use crate::{player::movement::DashTimer, screens::Screen};


const BULLET_TIME_DURATION: f32 = 2.0;
const BULLET_TIME_COOLDOWN: f32 = 1.0; // seconds
const BULLET_TIME_AURA_COST: f32 = 100.0;

pub fn reset_bullet_time(
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

pub fn go_into_bullet_time(
    real_time: Res<Time>,
    mut physics_time: ResMut<Time<Physics>>,
    mut player: Single<&mut Player>,
    audio: Res<Audio>,
    mut aura_event: EventWriter<AuraEarned>,
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
    player.aura_points -= BULLET_TIME_AURA_COST;
    aura_event.write(AuraEarned(-BULLET_TIME_AURA_COST));
}
