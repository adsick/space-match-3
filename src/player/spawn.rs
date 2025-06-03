use avian2d::prelude::{AngularDamping, AngularVelocity, LinearDamping, LinearVelocity, RigidBody};
use bevy::prelude::*;

use crate::screens::Screen;

use super::{
    Player,
    assets::PlayerAssets,
    movement::{MaxSpeed, MovementAcceleration, RotationSpeed},
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
        .add_systems(OnExit(Screen::Gameplay), despawn_player);
}

fn spawn_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    spawn_player_with_movement(
        &mut commands,
        Transform::from_xyz(0.0, 0.0, 0.0),
        player_assets,
    );
}

fn spawn_player_with_movement(
    commands: &mut Commands,
    transform: Transform,
    player_assets: Res<PlayerAssets>,
) -> Entity {
    let mut sprite = Sprite::from_image(player_assets.ship.clone());

    sprite.custom_size = Some(Vec2::new(1.0, 2.0));

    commands
        .spawn((
            Player,
            RigidBody::Dynamic,
            LinearVelocity::ZERO,
            AngularVelocity(0.0),
            MovementAcceleration(1000.0 / 100.0),
            AngularDamping(1.0),
            LinearDamping(0.3),
            RotationSpeed(2.0),
            MaxSpeed(1000.0 / 100.0),
            sprite,
            transform,
        ))
        .id()
}

fn despawn_player(mut commands: Commands, player_query: Single<Entity, With<Player>>) {
    commands.entity(*player_query).despawn();
}
