use std::time::Duration;

use avian2d::prelude::{Collider, CollisionEventsEnabled, CollisionStart, Physics, RigidBody};
use bevy::{
    color::palettes::css::WHITE,
    ecs::relationship::RelatedSpawnerCommands,
    mesh::{SphereKind, SphereMeshBuilder},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, Tween, TweenAnim,
    lens::{TransformRotationLens, TransformScaleLens},
};

use crate::{
    audio::AudioAssets,
    player::{Player, movement::AuraEarned},
    utils::{PointLightLens, StandardMaterialLens},
    vfx::ScreenShake,
};

const ASTEROID_SHADER_PATH: &str = "shaders/asteroid.wgsl";

pub fn plugin(app: &mut App) {
    app.add_plugins((MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, AsteroidMaterial>,
    >::default(),))
        .add_observer(on_add_asteroid)
        .add_observer(on_add_ship_asteroid_collider);
}

#[derive(Component)]
pub struct Asteroid {
    pub pos: Vec3,
    pub radius: f32,
}

#[derive(Component)]
pub struct ShipAsteroidCollider;

#[derive(Component)]
pub struct AnimatedExplosion;

fn on_add_asteroid(
    trigger: On<Add, Asteroid>,
    mut commands: Commands,
    asteroids: Query<&Asteroid>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut asteroid_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, AsteroidMaterial>>>,
) {
    let entity = trigger.event().event_target();
    let Ok(asteroid) = asteroids.get(entity) else {
        return;
    };

    commands.entity(entity).insert((
        RigidBody::Kinematic,
        // // CollisionLayers::new(
        // //     GameCollisionLayers::Meteorites,
        // //     GameCollisionLayers::Meteorites,
        // // ),
        Collider::circle(asteroid.radius * 0.85),
        Transform::from_translation(asteroid.pos),
        // .with_scale(Vec3::splat(meteorite_size)),
        Mesh3d(meshes.add(Sphere::new(asteroid.radius))),
        MeshMaterial3d(asteroid_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: WHITE.into(),
                // emissive: GREEN.into(),
                ..Default::default()
            },

            extension: AsteroidMaterial {
                terrain_seed: Vec4::new(rand::random::<f32>(), rand::random::<f32>(), 0.0, 0.0)
                    * 10.,
                radius: Vec4::splat(asteroid.radius),
            },
        })),
    ));
}

const ASTEROID_AURA_LOSS: f32 = 100.0;

fn on_add_ship_asteroid_collider(trigger: On<Add, ShipAsteroidCollider>, mut commands: Commands) {
    let entity = trigger.event().entity;

    commands
        .entity(entity)
        .insert(CollisionEventsEnabled)
        .observe(
            |trigger: On<CollisionStart>,
             mut commands: Commands,
             mut player: Single<&mut Player>,
             asteroids: Query<(&Asteroid, &Transform)>,

             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             mut screen_shake: ResMut<ScreenShake>,
             mut aura_event: MessageWriter<AuraEarned>,
             audio: Res<Audio>,
             audio_assets: Res<AudioAssets>,
             time: Res<Time<Physics>>| {
                let Ok((asteroid, asteroid_transform)) = asteroids.get(trigger.event().collider2)
                else {
                    return;
                };

                debug!("collision");

                audio.play(audio_assets.explosion.clone());

                commands.entity(trigger.event().collider2).despawn();

                player.aura_points = (player.aura_points - ASTEROID_AURA_LOSS).max(0.0);
                aura_event.write(AuraEarned(-ASTEROID_AURA_LOSS));
                player.near_asteroids = false;

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
                            Duration::from_millis(1200),
                        );
                        spawn_animated_explosion(
                            builder,
                            &mut meshes,
                            &mut materials,
                            asteroid.radius * 0.5,
                            Srgba::new(1.0, 0.7, 0.7, 1.0),
                            Srgba::new(1.0, 0.7, 0.7, 0.0),
                            Duration::from_millis(900),
                        );

                        spawn_animated_explosion(
                            builder,
                            &mut meshes,
                            &mut materials,
                            asteroid.radius * 0.2,
                            Srgba::new(1.0, 0.7, 0.7, 1.0),
                            Srgba::new(1.0, 0.9, 0.9, 0.6),
                            Duration::from_millis(700),
                        )
                    });

                screen_shake.until = time.elapsed_secs() + 0.5;
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

    let rotation_anim = Tween::new(
        EaseFunction::QuinticOut,
        duration,
        TransformRotationLens {
            start: Quat::IDENTITY,
            end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI * 6.),
        },
    );
    let scale_anim = Tween::new(
        EaseFunction::QuinticOut,
        duration,
        TransformScaleLens {
            start: Vec3::splat(1.0),
            end: Vec3::splat(5.5),
        },
    );

    let point_light_tween = Tween::new(
        EaseFunction::ExponentialIn,
        duration,
        PointLightLens {
            color_start: color_start.into(),
            color_end: color_end.into(),
            intensity_start: 10000000000000.,
            intensity_end: 0.,
        },
    );

    let color_tween = Tween::new(
        EaseFunction::QuinticOut,
        duration,
        StandardMaterialLens {
            color_start: color_start.into(),
            color_end: color_end.into(),
            emissive_start: (color_start * 10.).into(),
            emissive_end: (color_end * 10.).into(),
        },
    );

    let asteroid_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    let asteroid_material_id = asteroid_material.id();

    let anim_entity_id = builder
        .spawn((
            AnimatedExplosion,
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(asteroid_material),
            PointLight {
                color: color_start.into(),
                radius: 1000.,
                ..default()
            },
        ))
        .observe(|trigger: On<AnimCompletedEvent>, mut commands: Commands| {
            commands.entity(trigger.event().anim_entity).try_despawn();
        })
        .id();

    builder.spawn((
        TweenAnim::new(rotation_anim),
        AnimTarget::component::<Transform>(anim_entity_id),
    ));

    builder.spawn((
        TweenAnim::new(scale_anim),
        AnimTarget::component::<Transform>(anim_entity_id),
    ));

    builder.spawn((
        TweenAnim::new(point_light_tween),
        AnimTarget::component::<PointLight>(anim_entity_id),
    ));

    builder.spawn((
        TweenAnim::new(color_tween),
        AnimTarget::asset(asteroid_material_id),
    ));
}

fn foo(trigger: On<AnimCompletedEvent>, mut commands: Commands) {
    commands.entity(trigger.event().anim_entity).try_despawn();
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct AsteroidMaterial {
    #[uniform(100)]
    terrain_seed: Vec4,

    #[uniform(101)]
    radius: Vec4,
}

impl MaterialExtension for AsteroidMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        ASTEROID_SHADER_PATH.into()
    }

    // fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
    //     ASTEROID_SHADER_PATH.into()
    // }
}
