use avian2d::math::Scalar;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::PausableSystems;
use crate::screens::Screen;

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
    time: Res<Time>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let (mut velocity, mut angular_velocity, transform, acceleration, rotation_speed, max_speed) =
        player_query.into_inner();

    if left && right {
        angular_velocity.0 = 0.;
    } else if left {
        angular_velocity.0 = **rotation_speed;
    } else if right {
        angular_velocity.0 = -**rotation_speed;
    }

    if angular_velocity.0 != 0. {
        angular_velocity.0 *= 0.95;
    }

    let forward_dir = Vec2::new(
        transform.rotation.mul_vec3(Vec3::Y).x,
        transform.rotation.mul_vec3(Vec3::Y).y,
    );

    let thrust_force = forward_dir * **acceleration * 1. * time.delta_secs();

    velocity.x += thrust_force.x;
    velocity.y += thrust_force.y;

    let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if speed > **max_speed {
        let scale = **max_speed / speed;
        velocity.x *= scale;
        velocity.y *= scale;
    }
}

fn movement_update(
    player_query: Single<(&mut LinearVelocity, &MovementDampingFactor), With<Player>>,
    time: Res<Time>,
) {
    let (mut velocity, damping_factor) = player_query.into_inner();

    velocity.x *= 1.0 - **damping_factor * time.delta_secs();
    velocity.y *= 1.0 - **damping_factor * time.delta_secs();

    if velocity.x.abs() < 0.1 {
        velocity.x = 0.0;
    }
    if velocity.y.abs() < 0.1 {
        velocity.y = 0.0;
    }
}
