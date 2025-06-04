use avian2d::prelude::LinearVelocity;
use bevy::{ecs::query::QueryFilter, prelude::*};

pub mod assets;
pub mod movement;
pub mod spawn;
pub mod engine;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, spawn::plugin, assets::plugin, engine::plugin))
        .add_systems(Update, camera_follow_player);
}

#[derive(Component, QueryFilter)]
pub struct Player;

// TODO: ensure it runs in the right schedule
fn camera_follow_player(
    q_camera: Single<&mut Transform, With<Camera>>,
    q_player: Single<(&GlobalTransform, &LinearVelocity), With<Player>>,
    time: Res<Time>
) {
    let mut cam_transform = q_camera.into_inner();

    let (player_transform, vel) = q_player.into_inner();

    let vel_len = vel.0.length();

    cam_transform.translation = player_transform.translation().with_z(50.0);
    // cam_transform.translation = cam_transform.translation.move_towards(player_transform.translation().with_z(30.0 + 1.0 * vel_len), 45.0 * time.delta_secs());

    // cam_transform.translation = player_transform.translation().with_z(50.0 + 10.0 * vel_len);

    // cam_transform.translation.y += -vel_len / 4.0;

    // cam_transform.rotation = Quat::from_rotation_x(vel_len / 70.0);
}
