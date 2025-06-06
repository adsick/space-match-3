use avian2d::prelude::{LinearVelocity, Physics, PhysicsTime};
use bevy::{ecs::query::QueryFilter, prelude::*};

pub mod assets;
pub mod engine;
pub mod movement;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
        spawn::plugin,
        assets::plugin,
        engine::plugin,
    ))
    .add_systems(Update, (camera_follow_player, bullet_time));
}

#[derive(Component, QueryFilter)]
pub struct Player;

// TODO: ensure it runs in the right schedule
fn camera_follow_player(
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
        .lerp(200.0 + vel_len * 2.0, time.delta_secs());

    cam_transform.translation = player_transform.translation().with_z(z);
    // cam_transform.translation = cam_transform.translation.move_towards(player_transform.translation().with_z(30.0 + 1.0 * vel_len), 45.0 * time.delta_secs());

    // cam_transform.translation = player_transform.translation().with_z(50.0 + 10.0 * vel_len);

    // cam_transform.translation.y += -vel_len / 4.0;

    // cam_transform.rotation = Quat::from_rotation_x(vel_len / 70.0);
}

fn bullet_time(keys: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Physics>>) {
    if keys.pressed(KeyCode::Tab) {
        time.set_relative_speed(0.25);
    } else {
        time.set_relative_speed(1.0);
    }
}
