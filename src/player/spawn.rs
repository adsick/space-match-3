use avian2d::prelude::*;
use bevy::{color::palettes::css::VIOLET, prelude::*};

use crate::{
    asteroids::ShipAsteroidCollider,
    player::movement::{CurrentGas, DashTimer},
    screens::Screen,
};

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform = Transform::from_xyz(0.0, -1500.0, 0.0);

    commands
        .spawn((
            (
                Player {
                    // dash_timer: DashTimer(Timer::from_seconds(1., TimerMode::Once)),
                    ..Default::default()
                },
                Name::new("Player"),
                RigidBody::Dynamic,
                Collider::circle(3.),
                ShipAsteroidCollider {},
                LinearVelocity(Vec2::new(0., 900.)),
                AngularVelocity(0.0),
                Mass(1.0),
                AngularInertia(1.1),
                MovementAcceleration(8.0),
                GasBoost(92.0),
                CurrentGas(1.0),
                AngularDamping(12.0),
                LinearDamping(0.15),
                RotationSpeed(1000.0),
            ),
            (
                GravityScale(0.001),
                PointLight {
                    color: VIOLET.lighter(0.5).into(),
                    intensity: 100000000.,
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
            ),
        ))
        .with_children(|parent| {
            parent.spawn((engine::EngineFire {
                power: 0.5,
                color: Vec4::default(),
            },));

            parent
                .spawn((Collider::circle(50.0), Sensor, CollisionEventsEnabled))
                .observe(
                    |_trigger: Trigger<OnCollisionStart>, mut player: Single<&mut Player>| {
                        player.near_asteroids = true;
                    },
                )
                .observe(
                    |_trigger: Trigger<OnCollisionEnd>, mut player: Single<&mut Player>| {
                        player.near_asteroids = false;
                    },
                );
        });
}

fn despawn_player(mut commands: Commands, player_query: Single<Entity, With<Player>>) {
    commands.entity(*player_query).despawn();
}
