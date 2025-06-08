use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    asset::{Asset, Assets, Handle},
    color::{Alpha, palettes::css::RED},
    ecs::{entity::EntityHashSet, relationship::RelatedSpawnerCommands},
    math::{Vec3, Vec3Swizzles},
    pbr::{MeshMaterial3d, PointLight, StandardMaterial},
    prelude::*,
    time::Time,
    utils::default,
};
use bevy_spatial::{SpatialAccess, kdtree::KDTree2};
use bevy_tweening::{
    Animator, AssetAnimator, Tween, TweenCompleted, lens::TransformScaleLens,
};
use log::debug;

use crate::{
    player::Player, red_gas::{
        assets::RedOrbAssets, ExplosionDamage, PhysicalTimeAnimator, RedGasOrb, RedOrbExplosion, RedOrbExplosionEvent, RedOrbExplosionLens, EXPLOSION_CLEANUP_RADIUS, EXPLOSION_DURATION_SECS, MAX_EXPLOSION_RADIUS
    }, screens::Screen, utils::{PointLightLens, StandardMaterialLens}, Pause
};


pub fn on_add_explosive_gas_orb(
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

pub fn explode_orbs(
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

pub fn spawn_explosion_mesh(
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

pub fn spawn_explosion_light(builder: &mut RelatedSpawnerCommands<'_, ChildOf>) {
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

pub fn check_explosion_interactions(
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

        if explosion.interactions > 7 {
            entity_commands.try_despawn();
            continue;
        }

        let player_distance = player_transform
            .translation
            .xy()
            .distance_squared(explosion.pos);

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

pub fn update_component_animator_speed<T: Component>(
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

pub fn update_asset_animator_speed<T: Asset>(
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
