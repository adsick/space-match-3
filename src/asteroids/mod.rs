use std::{process::Child, time::Duration};

use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, OnCollisionStart, RigidBody,
};
use bevy::{
    app::App,
    asset::Assets,
    color::palettes::css::{GOLD, GREEN, RED},
    ecs::relationship::RelatedSpawnerCommands,
    math::{Quat, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        Commands, EaseFunction, Mesh, Mesh3d, MeshBuilder, OnAdd, Query, ResMut, Sphere, Transform,
        Trigger, *,
    },
    render::mesh::{SphereKind, SphereMeshBuilder},
};
use bevy_tweening::{
    AnimationSystem, Animator, AssetAnimator, BoxedTweenable, Lens, RepeatCount, RepeatStrategy,
    Targetable, Tracks, Tween, TweenCompleted, asset_animator_system, component_animator_system,
    lens::{ColorMaterialColorLens, TransformRotationLens, TransformScaleLens},
};

pub fn plugin(app: &mut App) {
    app.add_observer(on_add_asteroid)
        .add_observer(on_add_ship_asteroid_collider)
        .add_systems(
            Update,
            asset_animator_system::<StandardMaterial, MeshMaterial3d<StandardMaterial>>
                .in_set(AnimationSystem::AnimationUpdate),
        )
        .add_systems(
            Update,
            component_animator_system::<PointLight>.in_set(AnimationSystem::AnimationUpdate),
        );
}

#[derive(Component)]
pub struct Asteroid {
    pub pos: Vec3,
    pub radius: f32,
}

#[derive(Component)]
pub struct ShipAsteroidCollider {}

// pub fn meteorite_bundle(r: f32, pos: Vec2, parent: Entity) -> impl Bundle {
//     (
//         Meteorite { mass: r },
//         Transform::from_translation(
//             pos.extend((rand::random::<f32>() - 0.5) * METEORITE_CLOUD_Z_SCALE * r),
//         )
//         .with_scale(Vec3::splat(MIN_METEORITE_SIZE + METEORITE_SCALE * r)),
//         ChildOf(parent),
//     )
// }

fn on_add_asteroid(
    trigger: Trigger<OnAdd, Asteroid>,
    mut commands: Commands,
    asteroids: Query<&Asteroid>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut asteroid_materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    let Ok(asteroid) = asteroids.get(entity) else {
        return;
    };

    commands.entity(entity).insert((
        RigidBody::Kinematic,
        // CollisionLayers::new(
        //     GameCollisionLayers::Meteorites,
        //     GameCollisionLayers::Meteorites,
        // ),
        Collider::circle(asteroid.radius),
        Transform::from_translation(asteroid.pos),
        // .with_scale(Vec3::splat(meteorite_size)),
        Mesh3d(meshes.add(Sphere::new(asteroid.radius))),
        MeshMaterial3d(asteroid_materials.add(StandardMaterial {
            base_color: GREEN.into(),
            ..Default::default()
        })),
        // MeshMaterial3d(explosion_materials.add(ExtendedMaterial {
        //     base: StandardMaterial {
        //         alpha_mode: AlphaMode::Blend,
        //         ..Default::default()
        //     },
        //     extension: FireMaterialExtension::default(),
        // })),
    ));
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardMaterialColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}

impl Lens<StandardMaterial> for StandardMaterialColorLens {
    fn lerp(&mut self, target: &mut dyn Targetable<StandardMaterial>, ratio: f32) {
        target.base_color = self.start.mix(&self.end, ratio);
        target.emissive = self.start.mix(&self.end, ratio).into();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct PointLightLens {
    color_start: Color,
    color_end: Color,

    intensity_start: f32,
    intensity_end: f32,
}

impl Lens<PointLight> for PointLightLens {
    fn lerp(&mut self, target: &mut dyn Targetable<PointLight>, ratio: f32) {
        target.color = self.color_start.mix(&self.color_end, ratio);
        target.intensity = self.intensity_start.lerp(self.intensity_end, ratio);
    }
}

fn on_add_ship_asteroid_collider(
    trigger: Trigger<OnAdd, ShipAsteroidCollider>,
    mut commands: Commands,
) {
    let entity = trigger.target();

    commands
        .entity(entity)
        .insert((CollisionEventsEnabled,))
        .observe(
            |trigger: Trigger<OnCollisionStart>,
             mut commands: Commands,
             asteroids: Query<(&Asteroid, &Transform)>,

             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>| {
                // let meteorite = meteorites.get(trigger.collider);

                let Ok((asteroid, asteroid_transform)) = asteroids.get(trigger.collider) else {
                    return;
                };

                println!("collision");

                commands.entity(trigger.collider).despawn();

                // tween.with_completed

                commands
                    .spawn((
                        Transform::from_translation(asteroid_transform.translation),
                        Visibility::Visible,
                    ))
                    .with_children(|builder| {
                        spawn_animated_explosion(
                            builder,
                            &mut meshes,
                            &mut materials,
                            asteroid.radius,
                            Srgba::new(1.0, 0.7, 0.7, 1.0),
                            Srgba::new(1.0, 0.1, 0.1, 0.0),
                            Duration::from_millis(600),
                        );
                        spawn_animated_explosion(
                            builder,
                            &mut meshes,
                            &mut materials,
                            asteroid.radius * 0.6,
                            Srgba::new(1.0, 0.7, 0.7, 1.0),
                            Srgba::new(1.0, 0.7, 0.7, 0.0),
                            Duration::from_millis(500),
                        );

                        spawn_animated_explosion(
                            builder,
                            &mut meshes,
                            &mut materials,
                            asteroid.radius * 0.3,
                            Srgba::new(1.0, 0.7, 0.7, 1.0),
                            Srgba::new(1.0, 0.9, 0.9, 0.6),
                            Duration::from_millis(300),
                        )
                    });

                // let pressure_plate = trigger.target();
                // let other_entity = trigger.collider;
                // if player_query.contains(other_entity) {
                //     println!("Player {other_entity} stepped on pressure plate {pressure_plate}");
                // }
            },
        );
}

fn spawn_animated_explosion(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
    color_start: Srgba,
    color_end: Srgba,
    duration: Duration,
) {
    let sphere = SphereMeshBuilder::new(radius, SphereKind::Ico { subdivisions: 2 }).build();

    let transform_tween = Tracks::new([
        Tween::new(
            EaseFunction::QuinticOut,
            duration,
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI * 6.),
            },
        ),
        Tween::new(
            EaseFunction::QuinticOut,
            duration,
            TransformScaleLens {
                start: Vec3::splat(0.5),
                end: Vec3::splat(3.5),
            },
        ),
    ]);

    let point_light_tween = Tween::new(
        EaseFunction::ExponentialIn,
        duration,
        PointLightLens {
            color_start: color_start.into(),
            color_end: color_end.into(),
            intensity_start: 1000000000000.,
            intensity_end: 0.,
        },
    )
    .with_completed_event(0);

    let color_tween = Tween::new(
        EaseFunction::QuinticOut,
        duration,
        StandardMaterialColorLens {
            start: color_start.into(),
            end: color_end.into(),
        },
    )
    .with_completed_event(0);

    builder
        .spawn((
            Animator::new(transform_tween),
            Animator::new(point_light_tween),
            AssetAnimator::new(color_tween),
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color_start.into(),
                alpha_mode: AlphaMode::Blend,
                emissive: color_start.into(),
                ..Default::default()
            })),
            PointLight {
                color: color_start.into(),
                ..default()
            },
        ))
        .observe(|trigger: Trigger<TweenCompleted>, mut commands: Commands| {
            commands.entity(trigger.target()).despawn();
        });
}

// pub fn meteorite_collider_for_ship() -> impl Bundle {
//     (
//         RigidBody::Kinematic,
//         Collider::circle(10.),
//         CollisionLayers::new(
//             GameCollisionLayers::Meteorites,
//             GameCollisionLayers::Meteorites,
//         ),
//         CollisionEventsEnabled,
//         Observer::with_entity(self, entity)
//     )
// }
