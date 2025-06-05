use avian2d::prelude::{AngularDamping, AngularVelocity, LinearDamping, LinearVelocity, RigidBody};
use bevy::{
    color::palettes::css::{VIOLET, WHITE},
    prelude::*,
};

use crate::screens::Screen;

use super::{
    Player,
    assets::PlayerAssets,
    engine,
    movement::{GasBoost, MovementAcceleration, RotationSpeed},
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
        .add_systems(OnExit(Screen::Gameplay), despawn_player);
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    spawn_player_with_movement(&mut commands, transform, player_assets, materials);
}

fn spawn_player_with_movement(
    commands: &mut Commands,
    transform: Transform,
    player_assets: Res<PlayerAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Entity {
    commands
        .spawn((
            Player,
            RigidBody::Dynamic,
            LinearVelocity(Vec2::Y * 20.),
            AngularVelocity(0.0),
            MovementAcceleration(10.0),
            AngularDamping(1.0),
            LinearDamping(0.3),
            RotationSpeed(2.0),
            GasBoost(100.0),
            Mesh3d(player_assets.ship.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: VIOLET.into(),
                emissive: (VIOLET * 4.0).into(),
                ..Default::default()
            })),
            transform,
            children![(engine::EngineFire { power: 0.0 },)],
        ))
        .id()
}

fn despawn_player(mut commands: Commands, player_query: Single<Entity, With<Player>>) {
    commands.entity(*player_query).despawn();
}
