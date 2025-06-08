use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    app::{App, Update},
    asset::{Asset, AssetServer, Assets, Handle},
    color::{
        Alpha,
        palettes::css::RED,
    },
    ecs::{entity::EntityHashSet, relationship::RelatedSpawnerCommands},
    math::{NormedVectorSpace, Vec2, Vec3, Vec3Swizzles},
    pbr::{MeshMaterial3d, PointLight, StandardMaterial},
    prelude::{
        AlphaMode, ChildOf, Commands, Component, EaseFunction, Entity, Event, EventReader,
        EventWriter, FromWorld, IntoScheduleConfigs, Mesh, Mesh3d, MeshBuilder, OnAdd,
        Query, Res, ResMut, Resource, Single, State, StateScoped, Transform, Trigger, Visibility,
        With, in_state,
    },
    reflect::Reflect,
    render::mesh::{CircleMeshBuilder, SphereKind, SphereMeshBuilder},
    time::Time,
    utils::default,
};
use bevy_spatial::{
    AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode, kdtree::KDTree2,
};
use bevy_tweening::{
    AnimationSystem, Animator, AssetAnimator, Lens, Targetable, Tween, TweenCompleted,
    component_animator_system, lens::TransformScaleLens,
};
use log::debug;

use crate::{
    PausableSystems, Pause,
    asset_tracking::LoadResource,
    player::Player,
    screens::Screen,
    utils::{PointLightLens, StandardMaterialLens},
};

const MAX_EXPLOSION_RADIUS: f32 = 1500.;
const EXPLOSION_DURATION_SECS: u64 = 10;

pub fn plugin(app: &mut App) {
    app.add_plugins((AutomaticUpdate::<RedGasOrb>::new()
        .with_spatial_ds(SpatialStructure::KDTree2)
        .with_frequency(Duration::from_secs_f32(0.1))
        .with_transform(TransformMode::GlobalTransform),))
        .add_observer(on_add_explosive_gas_orb)
        .load_resource::<RedOrbAssets>()
        .insert_resource(ExplosionDamage(0.0))
        .add_event::<RedOrbExplosionEvent>()
        .add_systems(
            Update,
            explode_orbs
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        )
        .add_systems(
            Update,
            check_explosion_interactions
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        )
        .add_systems(
            Update,
            component_animator_system::<RedOrbExplosion>
                .in_set(AnimationSystem::AnimationUpdate)
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        )
        .add_systems(
            Update,
            update_component_animator_speed::<PointLight>.run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            Update,
            update_component_animator_speed::<RedOrbExplosion>.run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            Update,
            update_component_animator_speed::<Transform>.run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            Update,
            update_asset_animator_speed::<StandardMaterial>.run_if(in_state(Screen::Gameplay)),
        );
}

#[derive(Resource)]
/// When the damage reaches 1.0, the player must die.
pub struct ExplosionDamage(pub f32);

#[derive(Component)]
pub struct RedGasOrb {
    pub radius: f32,
    pub pos: Vec3,
}

#[derive(Component)]
struct RedOrbExplosion {
    radius: f32,
    pos: Vec2,
    // The number of other orbs this one has interacted with. For optimization purposes.
    interactions: usize,
}

#[derive(Resource, Asset, Clone, Reflect)]
struct RedOrbAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    explosion_mesh: Handle<Mesh>,
    // explosion_materials: Vec<Handle<StandardMaterial>>,
}

#[derive(Event)]
pub struct RedOrbExplosionEvent {
    pub entity: Entity,
}

#[derive(Component)]
struct PhysicalTimeAnimator {}

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

        // let explosion_materials = vec![
        //     assets.add(StandardMaterial {
        //         base_color: RED.with_alpha(0.5).into(),
        //         alpha_mode: AlphaMode::Blend,
        //         emissive: (RED * 1.0).into(),
        //         fog_enabled: false,
        //         ..Default::default()
        //     }),
        //     assets.add(StandardMaterial {
        //         base_color: RED.with_alpha(0.5).into(),
        //         alpha_mode: AlphaMode::Blend,
        //         emissive: (RED * 3.0).into(),
        //         fog_enabled: false,
        //         ..Default::default()
        //     }),
        // ];

        let explosion_mesh = assets.add(CircleMeshBuilder::new(1.0, 64).build());

        RedOrbAssets {
            mesh,
            material,
            // explosion_materials,
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

    // mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    orbs: Query<&RedGasOrb>,
    orb_assets: Res<RedOrbAssets>,
) {
    for event in events.read() {
        let Ok(orb) = orbs.get(event.entity) else {
            continue;
        };

        commands.entity(event.entity).try_despawn();

        let explosion_radius_tween = Tween::new(
            EaseFunction::SineOut,
            Duration::from_secs(EXPLOSION_DURATION_SECS),
            RedOrbExplosionLens {
                radius_start: orb.radius,
                radius_end: MAX_EXPLOSION_RADIUS,
            },
        )
        .with_completed_event(0);

        commands
            .spawn((
                StateScoped(Screen::Gameplay),
                RedOrbExplosion {
                    pos: orb.pos.xy(),
                    radius: orb.radius,
                    interactions: 0,
                },
                Transform::from_translation(orb.pos),
                PhysicalTimeAnimator {},
                Animator::new(explosion_radius_tween),
                Visibility::Visible,
            ))
            .with_children(|builder| {
                spawn_explosion_light(builder);
                spawn_explosion_mesh(builder, orb_assets.explosion_mesh.clone(), &mut materials);
            })
            .observe(|trigger: Trigger<TweenCompleted>, mut commands: Commands| {
                commands.entity(trigger.target()).try_despawn();
            });
    }
}

fn spawn_explosion_mesh(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,

    mesh: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let duration = Duration::from_secs(EXPLOSION_DURATION_SECS);

    let transform_tween = Tween::new(
        EaseFunction::SineOut,
        duration,
        TransformScaleLens {
            start: Vec3::splat(1.0),
            end: Vec3::splat(MAX_EXPLOSION_RADIUS),
        },
    );

    let color_start = RED;
    let color_end = RED.with_alpha(0.2);
    let color_tween = Tween::new(
        EaseFunction::SineOut,
        duration,
        StandardMaterialLens {
            color_start: color_start.into(),
            color_end: color_end.into(),
            emissive_start: (color_start * 10.).into(),
            emissive_end: (color_end * 4.).into(),
        },
    )
    .with_completed_event(0);

    builder.spawn((
        Animator::new(transform_tween),
        AssetAnimator::new(color_tween),
        PhysicalTimeAnimator {},
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            fog_enabled: false,
            ..Default::default()
        })),
    ));
    // .observe(|trigger: Trigger<TweenCompleted>, mut commands: Commands| {
    //     commands.entity(trigger.target()).try_despawn();
    // });
}

fn spawn_explosion_light(builder: &mut RelatedSpawnerCommands<'_, ChildOf>) {
    let duration = Duration::from_secs(3);

    let point_light_tween = Tween::new(
        EaseFunction::SineOut,
        duration,
        PointLightLens {
            color_start: RED.into(),
            color_end: RED.into(),
            intensity_start: 1000000000000000.,
            intensity_end: 0.,
        },
    )
    .with_completed_event(0);

    builder
        .spawn((
            PhysicalTimeAnimator {},
            Animator::new(point_light_tween),
            PointLight {
                color: RED.into(),
                radius: 5000.,
                ..default()
            },
        ))
        .observe(|trigger: Trigger<TweenCompleted>, mut commands: Commands| {
            commands.entity(trigger.target()).try_despawn();
        });
}

fn check_explosion_interactions(
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut RedOrbExplosion)>,
    red_orb_tree: Res<KDTree2<RedGasOrb>>,
    player_transform: Single<&Transform, With<Player>>,
    mut red_orb_explosion_events: EventWriter<RedOrbExplosionEvent>,

    mut explosion_damage: ResMut<ExplosionDamage>,

    time: Res<Time<Physics>>,
) {
    let mut i = 0;
    let mut is_inside_explosion = false;
    let mut already_exploded = EntityHashSet::default();

    for (entity, mut explosion) in &mut explosions {
        i += 1;
        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };

        if explosion.interactions > 20 {
            entity_commands.try_despawn();
            continue;
        }

        let player_distance = player_transform
            .translation
            .xy()
            .distance_squared(explosion.pos);

        const EXPLOSION_CLEANUP_RADIUS: f32 = 2000.;
        if player_distance > EXPLOSION_CLEANUP_RADIUS * EXPLOSION_CLEANUP_RADIUS {
            entity_commands.try_despawn();
            continue;
        }

        if player_distance < explosion.radius * explosion.radius {
            is_inside_explosion = true;
        }

        for (_, entity) in red_orb_tree.within_distance(explosion.pos, explosion.radius) {
            if let Some(entity) = entity {
                if already_exploded.contains(&entity) {
                    continue;
                }
                if commands.get_entity(entity).is_err() {
                    continue;
                }

                explosion.interactions += 1;
                red_orb_explosion_events.write(RedOrbExplosionEvent { entity });
                already_exploded.insert(entity);
            }
        }
    }

    if is_inside_explosion {
        explosion_damage.0 += time.delta_secs() / 3.0;
    } else {
        explosion_damage.0 = 0.;
    }

    debug!("explosion count: {i}");
}

fn update_component_animator_speed<T: Component>(
    animators: Query<&mut Animator<T>, With<PhysicalTimeAnimator>>,
    pause_state: Res<State<Pause>>,
    time: Res<Time<Physics>>,
) {
    let paused = pause_state.get().0;
    for mut animator in animators {
        if paused {
            animator.set_speed(0.0);
        } else {
            animator.set_speed(time.relative_speed());
        }
    }
}

fn update_asset_animator_speed<T: Asset>(
    animators: Query<&mut AssetAnimator<T>, With<PhysicalTimeAnimator>>,
    current_state: Res<State<Pause>>,
    time: Res<Time<Physics>>,
) {
    let paused = current_state.get().0;
    for mut animator in animators {
        if paused {
            animator.set_speed(0.0);
        } else {
            animator.set_speed(time.relative_speed());
        }
    }
}
