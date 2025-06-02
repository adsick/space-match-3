use bevy::{ecs::query::QueryFilter, prelude::*};

pub mod assets;
pub mod movement;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, spawn::plugin, assets::plugin))
        .add_systems(Update, camera_follow_player);
}

#[derive(Component, QueryFilter)]
pub struct Player;

fn camera_follow_player(
    mut q_camera: Query<&mut Transform, With<Camera>>,
    q_player: Query<&GlobalTransform, With<Player>>,
) {
    let Ok(mut cam_transform) = q_camera.single_mut() else {
        return;
    };

    if let Ok(player_transform) = q_player.single() {
        cam_transform.translation = player_transform
            .translation()
            .truncate()
            .extend(cam_transform.translation.z);
    }
}
