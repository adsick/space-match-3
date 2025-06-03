use avian2d::math::Scalar;
use avian2d::prelude::*;
use bevy::prelude::*;

// use bevy::diagnostic::{DiagnosticPath, DiagnosticsStore};

use crate::PausableSystems;
use crate::screens::Screen;
use crate::terrain::TerrainGenerator;

use super::Player;

#[derive(Component, Deref, DerefMut)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Component, Deref, DerefMut)]
pub struct MovementDampingFactor(pub Scalar);

#[derive(Component, Deref, DerefMut)]
pub struct RotationSpeed(pub Scalar);

#[derive(Component, Deref, DerefMut)]
pub struct MaxSpeed(pub Scalar);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (keyboard_input, movement_update)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Single<
        (
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Transform,
            &MovementAcceleration,
            &RotationSpeed,
            &MaxSpeed,
        ),
        With<Player>,
    >,
    // diagnostics: Res<DiagnosticsStore>,
    terrain: Res<TerrainGenerator>,
    time: Res<Time>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let brake = keyboard_input.pressed(KeyCode::Space);

    let (mut velocity, mut angular_velocity, transform, acceleration, rotation_speed, _max_speed) =
        player_query.into_inner();

    if left && right {
        angular_velocity.0 = 0.; // *instantly stops rotation when both keys are pressed
    } else if left {
        angular_velocity.0 = **rotation_speed;
    } else if right {
        angular_velocity.0 = -**rotation_speed;
    }

    let forward_dir = (transform.rotation * Vec3::Y).truncate();

    let thrust_force = forward_dir * **acceleration;

    velocity.0 += thrust_force
        * time.delta_secs()
        * (terrain
            .orb_probability(transform.translation.truncate())
            .clamp(0.1, 0.25)
            * 6.0)
        * (!brake as i32 as f32).clamp(0.15, 1.0);

    // TODO: not framerate-independent
    let speed = velocity.0.length();
    debug!("{speed:.2}");

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
}

#[allow(unused)]
fn movement_update(
    mut player_query: Single<(&mut LinearVelocity, &Transform), With<Player>>,
    time: Res<Time>,
) {
    // info!("going thru orb");
    // player_query.0.x *= 1.01;
    // player_query.0.y *= 1.01;

    // velocity.x *= 1.0 * damping_factor.0.powf(time.delta_secs());
    // velocity.y *= 1.0 * damping_factor.0.powf(time.delta_secs());

    // if velocity.x.abs() < 0.001 {
    //     velocity.x = 0.0;
    // }
    // if velocity.y.abs() < 0.001 {
    //     velocity.y = 0.0;
    // }
}
