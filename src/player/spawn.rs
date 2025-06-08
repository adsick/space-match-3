use avian2d::prelude::*;
use bevy::{color::palettes::css::VIOLET, prelude::*};

use crate::{asteroids::ShipAsteroidCollider, player::movement::CurrentGas, screens::Screen};

use super::{
    Player,
    assets::PlayerAssets,
    engine,
    movement::{GasBoost, MovementAcceleration, PlayerControls, RotationSpeed},
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
                Player::default(),
                Name::new("Player"),
                RigidBody::Dynamic,
                Collider::circle(3.),
                ShipAsteroidCollider {},
                LinearVelocity(Vec2::new(0., 900.)),
                AngularVelocity(0.0),
                Mass(1.0),
                AngularInertia(1.0),
                MovementAcceleration(20.0),
                GasBoost(10.0),
                CurrentGas(1.0),
                AngularDamping(10.0),
                LinearDamping(0.2),
                RotationSpeed(280.0),
            ),
            (
                GravityScale(0.001),
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
            ),
        ))
        .with_children(|child| {
            child
                .spawn((
                    engine::EngineFire {
                        power: 0.5,
                        color: Vec4::default(),
                    },
                    Collider::circle(50.0),
                    Sensor,
                    CollisionEventsEnabled,
                ))
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
