use avian2d::math::Scalar;
use avian2d::prelude::*;
use bevy::color::palettes::css::GREEN_YELLOW;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl as _};

// use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};

use crate::PausableSystems;
use crate::audio::AudioAssets;
use crate::player::Score;
use crate::screens::Screen;
use crate::space::GasGenerator;
use crate::space::gas::ignite_gas;

use super::Player;

#[derive(Component, Deref, DerefMut, Reflect, Default)]
pub struct DashTimer(pub Timer);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct RotationSpeed(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct GasBoost(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct CurrentGas(pub Scalar);

pub const GLIDE_FORCE: f32 = 330.0;
// pub const DRAG_FORCE: f32 = 0.05;
pub const SPEED_LOCK_IN: f32 = 21.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementAcceleration>()
        .register_type::<RotationSpeed>()
        .register_type::<GasBoost>()
        .register_type::<CurrentGas>()
        .add_event::<AuraEarned>()
        .insert_resource(PlayerControls { enabled: false })
        .add_systems(
            FixedUpdate,
            (thrust.after(ignite_gas), glide)
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        );
}

#[derive(Event)]
pub struct AuraEarned(pub f32);

#[derive(Resource)]
pub struct PlayerControls {
    pub enabled: bool,
}

// *maybe rename this function
pub fn thrust(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Single<
        (
            &mut Player,
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut CurrentGas,
            &Rotation,
            &LinearVelocity,
            &MovementAcceleration,
            &RotationSpeed,
            &GasBoost,
        ),
        With<Player>,
    >,
    player_controls: Res<PlayerControls>,
    time: Res<Time<Physics>>,
    mut score: ResMut<Score>,
    mut aura_event: EventWriter<AuraEarned>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let brake = keyboard_input.pressed(KeyCode::KeyS);

    let (
        mut player,
        mut force,
        mut torque,
        mut current_gas,
        rotation,
        velocity,
        acceleration,
        rotation_speed,
        gas_boost,
    ) = player_query.into_inner();

    let vel_length = velocity.length();
    let delta = time.delta_secs();
    score.0 += vel_length / 250.0 * delta;

    let earned = vel_length * vel_length / 350.0 * delta * player.near_asteroids as u32 as f32;

    if earned > 0.0 {
        player.aura_points += earned;
        aura_event.write(AuraEarned(earned));
        debug!("earned: {earned}");

        audio.play(audio_assets.pop_3.clone()).with_volume(0.03);
    }
    player.aura_points += 1.0 * delta;

    player.aura_points = player.aura_points.max(0.0);

    force.persistent = false;
    torque.persistent = false;

    let speed_sqrt = vel_length.sqrt();
    debug!("sqrt(speed) = {speed_sqrt:.2}",);
    let tq = rotation_speed.0 / speed_sqrt.max(SPEED_LOCK_IN);
    if left && player_controls.enabled {
        torque.apply_torque(tq);
    }
    if right && player_controls.enabled {
        torque.apply_torque(-tq);
    }

    let forward_dir = rotation * Vec2::Y;

    let mut thrust_force = forward_dir * **acceleration;

    if brake && player_controls.enabled {
        thrust_force *= 0.15;
    }

    let gas_boost_force = forward_dir * current_gas.0 * gas_boost.0;

    force.apply_force(thrust_force);
    force.apply_force(gas_boost_force);

    let before = current_gas.0;
    current_gas.0 *= 0.01f32.powf(delta);
    debug!("current_gas: {before:.2} -> {:.2}", current_gas.0);
}

fn glide(
    player_query: Single<(&LinearVelocity, &mut ExternalForce, &Transform), With<Player>>,
    // mut gizmos: Gizmos,
    gas: Res<GasGenerator>,
) {
    let (linvel, mut force, ship_tr) = player_query.into_inner();

    let forward = ship_tr.up().truncate();
    let ship_pos = ship_tr.translation.truncate();

    // let side_vel = linvel.0 - linvel.0.project_onto(forward);

    let amount = gas.sample(ship_pos).clamp(0.0, 1.0) + 0.3;

    // let drag = -side_vel * amount * DRAG_FORCE;
    let glide = (forward - forward.project_onto(linvel.0)) * amount * GLIDE_FORCE; // basically we want to rotate the linvel by applying a perpendicular force...

    // gizmos.ray_2d(ship_pos, drag * 1.0, RED);
    // gizmos.ray_2d(ship_pos, glide * 1.0, GREEN_YELLOW);

    force.apply_force(glide);
}
