use avian2d::math::Scalar;
use avian2d::prelude::*;
use bevy::prelude::*;

// use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};

use crate::PausableSystems;
use crate::gas::pickup_gas;
use crate::screens::Screen;
use crate::space::{GasGenerator, orb_explosion::OrbExplosion};

use super::Player;

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct RotationSpeed(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct GasBoost(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct CurrentGas(pub Scalar);

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementAcceleration>()
        .register_type::<RotationSpeed>()
        .register_type::<GasBoost>()
        .register_type::<CurrentGas>()
        .add_systems(
            Update,
            (thrust.after(pickup_gas), glide)
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        );
}

#[derive(Component)]
pub struct PlayerControlls {
    pub enabled: bool,
}

// *maybe rename this function
fn thrust(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Single<
        (
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut CurrentGas,
            &Transform,
            &Rotation,
            &LinearVelocity,
            &MovementAcceleration,
            &RotationSpeed,
            &GasBoost,
            &PlayerControlls,
        ),
        With<Player>,
    >,
    time: Res<Time<Physics>>,
    gas: Res<GasGenerator>,
    mut expl_ev: EventWriter<OrbExplosion>, // gas_orb_query: Query<(Entity, &Transform), With<GasOrb>>,
                                            // diagnostics: Res<DiagnosticsStore>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let brake = keyboard_input.pressed(KeyCode::Space);

    let (
        mut force,
        mut torque,
        mut current_gas,
        transform,
        rotation,
        velocity,
        acceleration,
        rotation_speed,
        gas_boost,
        controlls,
    ) = player_query.into_inner();

    force.persistent = false;
    torque.persistent = false;
    let tq = rotation_speed.0 / velocity.length().max(100.0);
    if left && controlls.enabled {
        torque.apply_torque(tq);
    }
    if right && controlls.enabled {
        torque.apply_torque(-tq);
    }

    let forward_dir = rotation * Vec2::Y;

    let mut thrust_force = forward_dir * **acceleration;

    if brake && controlls.enabled {
        thrust_force *= 0.15;
    }

    let gas_boost_force = forward_dir * current_gas.0 * gas_boost.0;

    force.apply_force(thrust_force);
    force.apply_force(gas_boost_force); // TODO: test this properly

    let gas_density = gas.sample(transform.translation.truncate());

    if gas_density > 0.0 {
        let player_pos = transform.translation.truncate();

        expl_ev.write(OrbExplosion { pos: player_pos });
    }

    // velocity.0 += (thrust_force + gas_boost) * time.delta_secs();

    // TODO: not framerate-independent
    // let speed = velocity.0.length();
    // debug!("{speed:.2}");

    // we don't need additional speed limiting as avian's dampening will do it for us anyway
    // if speed > **max_speed {
    //     let fps = diagnostics
    //         .get(&DiagnosticPath::const_new("fps"))
    //         .and_then(|fps| fps.smoothed())
    //         .unwrap_or(60.0);

    //     Dir2::try_from(velocity.0)
    //         .map(|dir| {
    //             velocity.0 = velocity.lerp(dir * max_speed.0, time.delta_secs() * fps as f32 / 1000.0);
    //         })
    //         .ok();
    // }
    current_gas.0 *= 0.01f32.powf(time.delta_secs());
}

fn glide(
    player_query: Single<(&LinearVelocity, &mut ExternalForce, &Transform), With<Player>>,
    // mut gizmos: Gizmos,
    gas: Res<GasGenerator>,
) {
    let (linvel, mut force, transform) = player_query.into_inner();

    let forward_dir = (transform.rotation * Vec3::Y).truncate();

    let drag_angle = 1.0 - forward_dir.dot(linvel.normalize_or_zero()).abs();

    let amount = gas.sample(transform.translation.truncate()).clamp(0.0, 1.0);

    let drag_force = -linvel.0 * drag_angle * amount * 5.0;
    let glide_force =
        drag_force.length() * forward_dir.dot(linvel.normalize_or_zero()).signum() * forward_dir;

    // gizmos.ray_2d(
    //     transform.translation.xy(),
    //     (drag_force + glide_force) * 10.0,
    //     bevy::color::palettes::css::LIGHT_CYAN,
    // );
    force.apply_force(drag_force);
    force.apply_force(glide_force);
}
