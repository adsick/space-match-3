use avian2d::prelude::*;
use bevy::{color::palettes::css::VIOLET, prelude::*};

use crate::{
    asteroids::ShipAsteroidCollider,
    player::{Powers, movement::CurrentGas},
    screens::Screen,
};

use super::{
    Player,
    assets::PlayerAssets,
    engine,
    movement::{GasBoost, MovementAcceleration, PlayerControlls, RotationSpeed},
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
        .add_systems(OnExit(Screen::Gameplay), despawn_player);
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform = Transform::from_xyz(0.0, -1500.0, 0.0);
    commands.spawn((
        (
            Player,
            Name::new("Player"),
            RigidBody::Dynamic,
            Collider::circle(5.),
            ShipAsteroidCollider {},
            LinearVelocity(Vec2::new(0., 600.)),
            AngularVelocity(0.0),
            Mass(1.0),
            AngularInertia(1.0),
            MovementAcceleration(20.0),
            GasBoost(50.0),
            CurrentGas(1.0),
            AngularDamping(10.0),
            LinearDamping(0.3),
            RotationSpeed(2000.0),
        ),
        (
            PlayerControlls { enabled: true },
            Powers {
                style_points: 0,
                bullet_time_until: 0.0,
                bullet_time_cooldown_until: 0.0,
            },
            PointLight {
                color: VIOLET.into(),
                intensity: 1000000000.,
                range: 400.,

                ..default()
            },
            Mesh3d(player_assets.ship.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: VIOLET.into(),
                emissive: (VIOLET * 4.0).into(),
                ..Default::default()
            })),
            transform,
            children![(engine::EngineFire {
                power: 0.5,
                color: Vec4::default()
            },),],
        ),
    ));
}

fn despawn_player(mut commands: Commands, player_query: Single<Entity, With<Player>>) {
    commands.entity(*player_query).despawn();
}
