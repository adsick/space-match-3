use avian2d::math::Scalar;
use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN_YELLOW, RED};
use bevy::prelude::*;

// use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};

use crate::PausableSystems;
use crate::screens::Screen;
use crate::space::GasGenerator;
use crate::space::gas::ignite_gas;

use super::Player;

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct RotationSpeed(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct GasBoost(pub Scalar);

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct CurrentGas(pub Scalar);

pub const GLIDE_FORCE: f32 = 30.0;
pub const DRAG_FORCE: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementAcceleration>()
        .register_type::<RotationSpeed>()
        .register_type::<GasBoost>()
        .register_type::<CurrentGas>()
        .add_systems(
            Update,
            (thrust.after(ignite_gas), glide)
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
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let brake = keyboard_input.pressed(KeyCode::KeyS);

    let (
        mut force,
        mut torque,
        mut current_gas,
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

    let before = current_gas.0;
    current_gas.0 *= 0.01f32.powf(time.delta_secs());
    debug!("current_gas: {before:.2} -> {:.2}", current_gas.0);
}

fn glide(
    player_query: Single<(&LinearVelocity, &mut ExternalForce, &Transform), With<Player>>,
    mut gizmos: Gizmos,
    gas: Res<GasGenerator>,
) {
    let (linvel, mut force, ship_tr) = player_query.into_inner();

    let forward = ship_tr.up().truncate();
    let ship_pos = ship_tr.translation.truncate();

    let side_vel = linvel.0 - linvel.0.project_onto(forward);

    let amount = gas.sample(ship_pos).clamp(0.0, 1.0);

    let drag = -side_vel * amount * DRAG_FORCE;
    let glide = (forward - forward.project_onto(linvel.0)) * amount * GLIDE_FORCE; // basically we want to rotate the linvel by applying a perpendicular force...

    gizmos.ray_2d(ship_pos, drag * 5.0, RED);
    gizmos.ray_2d(ship_pos, glide * 5.0, GREEN_YELLOW);

    force.apply_force(drag + glide);
}
