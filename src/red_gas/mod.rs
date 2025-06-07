use std::time::Duration;

use bevy::{
    app::{App, Update},
    asset::{Asset, AssetServer, Assets, Handle},
    color::{
        Alpha,
        palettes::css::{GREEN, RED, WHEAT, WHITE},
    },
    math::{NormedVectorSpace, Vec2, Vec3, Vec3Swizzles},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Commands, Component, EaseFunction, Entity, Event, EventReader, EventWriter,
        FromWorld, IntoScheduleConfigs, Mesh, Mesh3d, MeshBuilder, OnAdd, Query, Res, ResMut,
        Resource, Single, Transform, Trigger, With,
    },
    reflect::Reflect,
    render::mesh::{CircleMeshBuilder, SphereKind, SphereMeshBuilder},
};
use bevy_spatial::{
    AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode, kdtree::KDTree2,
};
use bevy_tweening::{
    AnimationSystem, Animator, Lens, Targetable, Tween, TweenCompleted, component_animator_system,
    lens::TransformScaleLens,
};

use crate::{
    PhysicsLayers,
    player::{self, Player},
};

pub fn plugin(app: &mut App) {
    app.add_plugins((AutomaticUpdate::<RedGasOrb>::new()
        .with_spatial_ds(SpatialStructure::KDTree2)
        .with_frequency(Duration::from_secs_f32(0.1))
        .with_transform(TransformMode::GlobalTransform),))
        .add_observer(on_add_explosive_gas_orb)
        .init_resource::<RedOrbAssets>()
        .add_event::<RedOrbExplosionEvent>()
        .add_systems(Update, explode_orbs)
        .add_systems(Update, check_explosion_interactions)
        .add_systems(
            Update,
            component_animator_system::<RedOrbExplosion>.in_set(AnimationSystem::AnimationUpdate),
        );
}

#[derive(Component)]
pub struct RedGasOrb {
    pub radius: f32,
    pub pos: Vec3,
}

#[derive(Component)]
struct RedOrbExplosion {
    pub radius: f32,
    pub pos: Vec2,
}

#[derive(Resource, Asset, Clone, Reflect)]
struct RedOrbAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    explosion_mesh: Handle<Mesh>,
    explosion_materials: Vec<Handle<StandardMaterial>>,
}

#[derive(Event)]
pub struct RedOrbExplosionEvent {
    pub entity: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct RedOrbExplosionLens {
    radius_start: f32,
    radius_end: f32,
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

impl Lens<RedOrbExplosion> for RedOrbExplosionLens {
    fn lerp(&mut self, target: &mut dyn Targetable<RedOrbExplosion>, ratio: f32) {
        target.radius = mix(self.radius_start, self.radius_end, ratio);
    }
}

impl FromWorld for RedOrbAssets {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let assets = world.resource::<AssetServer>();

        let mesh =
            assets.add(SphereMeshBuilder::new(1.0, SphereKind::Ico { subdivisions: 3 }).build());

        let material = assets.add(StandardMaterial {
            base_color: RED.with_alpha(0.7).into(),
            alpha_mode: AlphaMode::Blend,
            emissive: (RED * 5.0).into(),
            ..Default::default()
        });

        let explosion_materials = vec![
            assets.add(StandardMaterial {
                base_color: RED.with_alpha(0.5).into(),
                alpha_mode: AlphaMode::Blend,
                emissive: (RED * 1.0).into(),
                ..Default::default()
            }),
            assets.add(StandardMaterial {
                base_color: RED.with_alpha(0.5).into(),
                alpha_mode: AlphaMode::Blend,
                emissive: (RED * 3.0).into(),
                ..Default::default()
            }),
        ];

        let explosion_mesh = assets.add(CircleMeshBuilder::new(1.0, 12).build());

        RedOrbAssets {
            mesh,
            material,
            explosion_materials,
            explosion_mesh,
        }
    }
}

fn on_add_explosive_gas_orb(
    trigger: Trigger<OnAdd, RedGasOrb>,
    mut commands: Commands,
    orbs: Query<&RedGasOrb>,
    orb_assets: Res<RedOrbAssets>,
) {
    let entity = trigger.target();
    let Ok(orb) = orbs.get(entity) else {
        return;
    };

    let mut transform = Transform::from_translation(orb.pos).with_scale(Vec3::splat(orb.radius));

    transform.rotate_local_z(orb.pos.x);
    transform.rotate_local_x(orb.pos.y);

    commands.entity(entity).insert((
        transform,
        Mesh3d(orb_assets.mesh.clone()),
        MeshMaterial3d(orb_assets.material.clone()),
    ));
}

fn explode_orbs(
    mut events: EventReader<RedOrbExplosionEvent>,
    mut commands: Commands,
    orbs: Query<&RedGasOrb>,

    orb_assets: Res<RedOrbAssets>,
) {
    for event in events.read() {
        let Ok(orb) = orbs.get(event.entity) else {
            continue;
        };

        commands.entity(event.entity).try_despawn();

        const MAX_EXPLOSION_RADIUS: f32 = 500.;
        const EXPLOSION_DURATION_SECS: u64 = 10;

        let explosion_radius_tween = Tween::new(
            EaseFunction::QuinticOut,
            Duration::from_secs(EXPLOSION_DURATION_SECS),
            RedOrbExplosionLens {
                radius_start: orb.radius,
                radius_end: MAX_EXPLOSION_RADIUS,
            },
        );

        let explosion_sphere_tween = Tween::new(
            EaseFunction::QuinticOut,
            Duration::from_secs(EXPLOSION_DURATION_SECS),
            TransformScaleLens {
                start: Vec3::splat(orb.radius),
                end: Vec3::splat(MAX_EXPLOSION_RADIUS),
            },
        )
        .with_completed_event(0);

        commands
            .spawn((
                RedOrbExplosion {
                    pos: orb.pos.xy(),
                    radius: orb.radius,
                },
                Mesh3d(orb_assets.explosion_mesh.clone()),
                MeshMaterial3d(orb_assets.explosion_materials[0].clone()),
                Transform::from_translation(orb.pos),
                Animator::new(explosion_radius_tween),
                Animator::new(explosion_sphere_tween),
            ))
            .observe(|trigger: Trigger<TweenCompleted>, mut commands: Commands| {
                commands.entity(trigger.target()).try_despawn();
            });
    }
}

fn check_explosion_interactions(
    mut commands: Commands,
    explosions: Query<(Entity, &RedOrbExplosion)>,
    red_orb_tree: Res<KDTree2<RedGasOrb>>,
    player_transform: Single<&Transform, With<Player>>,
    mut red_orb_explosion_events: EventWriter<RedOrbExplosionEvent>,
) {
    for (entity, explosion) in explosions {
        const EXPLOSION_CLEANUP_RADIUS: f32 = 1500.;
        if player_transform
            .translation
            .xy()
            .distance_squared(explosion.pos)
            > EXPLOSION_CLEANUP_RADIUS * EXPLOSION_CLEANUP_RADIUS
        {
            commands.entity(entity).try_despawn();
            continue;
        }

        for (_, entity) in red_orb_tree.within_distance(explosion.pos, explosion.radius) {
            if let Some(e) = entity {
                red_orb_explosion_events.write(RedOrbExplosionEvent { entity: e });
            }
        }
    }
}
