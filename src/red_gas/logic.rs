use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    asset::Handle,
    color::palettes::css::{RED, YELLOW},
    ecs::{entity::EntityHashSet, relationship::RelatedSpawnerCommands},
    light::{NotShadowCaster, NotShadowReceiver},
    math::{Vec3, Vec3Swizzles},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
    time::Time,
    utils::default,
};
use bevy_spatial::{SpatialAccess, kdtree::KDTree2};
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, lens::TransformScaleLens};
use log::debug;

use crate::{
    Pause,
    player::Player,
    red_gas::{
        EXPLOSION_DURATION_SECS, ExplosionDamage, MAX_EXPLOSION_RADIUS, PhysicalTimeAnimator,
        RedGasOrb, RedOrbExplosion, RedOrbExplosionEvent, RedOrbExplosionLens,
        assets::RedOrbAssets,
    },
    screens::Screen,
    space::intro::IntroState,
    utils::PointLightLens,
};

pub fn on_add_explosive_gas_orb(
    trigger: On<Add, RedGasOrb>,
    mut commands: Commands,
    orbs: Query<&RedGasOrb>,
    orb_assets: Res<RedOrbAssets>,
) {
    let entity = trigger.event().event_target();
    let Ok(orb) = orbs.get(entity) else {
        return;
    };

    let transform = Transform::from_translation(orb.pos).with_scale(Vec3::splat(orb.radius));

    commands.entity(entity).insert((
        transform,
        Mesh3d(orb_assets.mesh.clone()),
        MeshMaterial3d(orb_assets.material.clone()),
    ));
}

pub fn explode_red_orbs(
    mut events: MessageReader<RedOrbExplosionEvent>,
    mut commands: Commands,

    orbs: Query<&RedGasOrb>,
    orb_assets: Res<RedOrbAssets>,

    // debug
    player: Single<&Transform, With<Player>>,
    mut gizmo: Gizmos,
) {
    for event in events.read() {
        let Ok(orb) = orbs.get(event.entity) else {
            continue;
        };

        commands.entity(event.entity).try_despawn();

        // debug

        let ship_tr = player.translation.truncate();
        let source = event.meta;

        let pos = orb.pos.truncate();
        let relative = pos - ship_tr;

        if relative.dot(player.up().truncate()) > 0.0 {
            debug!("Explosion in front. Source: {source}");
            gizmo.line_2d(ship_tr, pos, RED);
            gizmo.circle_2d(Isometry2d::from_translation(pos), 10.0, RED);
        } else {
            gizmo.line_2d(ship_tr, pos, YELLOW);
        }

        // debug end

        let explosion_radius_tween = Tween::new(
            EaseFunction::SineOut,
            Duration::from_secs(EXPLOSION_DURATION_SECS),
            RedOrbExplosionLens {
                radius_start: orb.radius,
                radius_end: MAX_EXPLOSION_RADIUS,
            },
        );

        commands
            .spawn((
                DespawnOnExit(Screen::Gameplay),
                RedOrbExplosion {
                    pos: orb.pos.xy(),
                    radius: orb.radius,
                    interactions: 0,
                },
                Transform::from_translation(orb.pos),
                PhysicalTimeAnimator {},
                TweenAnim::new(explosion_radius_tween),
                Visibility::Visible,
            ))
            .with_children(|builder| {
                spawn_explosion_light(builder);
                spawn_explosion_mesh(
                    builder,
                    orb_assets.explosion_mesh.clone(),
                    orb_assets.explosion_material.clone(),
                );
            })
            .observe(|trigger: On<AnimCompletedEvent>, mut commands: Commands| {
                commands.entity(trigger.event_target()).try_despawn();
            });
    }
}

pub fn spawn_explosion_mesh(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,

    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
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

    builder
        .spawn((
            TweenAnim::new(transform_tween),
            PhysicalTimeAnimator {},
            Mesh3d(mesh),
            MeshMaterial3d(material),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .observe(|trigger: On<AnimCompletedEvent>, mut commands: Commands| {
            commands
                .entity(trigger.event().event_target())
                .try_despawn();
        });
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
    );
    // .with_completed_event(0);

    builder
        .spawn((
            PhysicalTimeAnimator {},
            TweenAnim::new(point_light_tween),
            PointLight {
                color: RED.into(),
                radius: 5000.,
                ..default()
            },
        ))
        .observe(|trigger: On<AnimCompletedEvent>, mut commands: Commands| {
            commands
                .entity(trigger.event().event_target())
                .try_despawn();
        });
}

pub fn check_explosion_interactions(
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut RedOrbExplosion)>,
    red_orb_tree: Res<KDTree2<RedGasOrb>>,
    player_transform: Single<&Transform, With<Player>>,
    mut red_orb_explosion_events: MessageWriter<RedOrbExplosionEvent>,

    mut explosion_damage: ResMut<ExplosionDamage>,

    intro_state: Res<State<IntroState>>,

    time: Res<Time<Physics>>,
) {
    let mut i = 0;
    let mut is_inside_explosion = false;
    let mut already_exploded = EntityHashSet::default();

    let player_pos = player_transform.translation.xy();

    let mut explosion_distances = Vec::<(f32, Entity)>::new();

    // let mut closest_explotions =

    for (entity, mut explosion) in &mut explosions {
        i += 1;

        // if explosion.interactions > 7 {
        //     let Ok(mut entity_commands) = commands.get_entity(entity) else {
        //         continue;
        //     };
        //     entity_commands.try_despawn();
        //     continue;
        // }

        let player_distance = player_pos.distance_squared(explosion.pos);

        if player_distance < explosion.radius * explosion.radius {
            is_inside_explosion = true;
        }

        explosion_distances.push((player_distance, entity));

        // if player_distance > EXPLOSION_CLEANUP_RADIUS * EXPLOSION_CLEANUP_RADIUS {
        //     entity_commands.try_despawn();
        //     continue;
        // }

        for (_, entity) in red_orb_tree.within_distance(explosion.pos, explosion.radius) {
            if let Some(entity) = entity {
                if already_exploded.contains(&entity) {
                    continue;
                }
                if commands.get_entity(entity).is_err() {
                    continue;
                }

                explosion.interactions += 1;
                red_orb_explosion_events.write(RedOrbExplosionEvent { entity, meta: 1 });
                already_exploded.insert(entity);
            }
        }
    }

    explosion_distances.sort_by(|a, b| a.0.total_cmp(&b.0));

    // println!("explosion distances: {:?}", explosion_distances);
    let max_explosion_count = if intro_state.0 { 60 } else { 20 };
    for (_, entity) in explosion_distances.iter().skip(max_explosion_count) {
        commands.entity(*entity).try_despawn();
    }

    if is_inside_explosion {
        explosion_damage.0 += time.delta_secs() / 2.0;
        // explosion_damage.0 += time.delta_secs() / 2.0;
    } else {
        explosion_damage.0 = 0.;
    }

    // debug!("explosion count: {i}");
}

pub fn update_component_animator_speed(
    animators: Query<&mut TweenAnim, With<PhysicalTimeAnimator>>,
    pause_state: Res<State<Pause>>,
    time: Res<Time<Physics>>,
) {
    let paused = pause_state.get().0;
    for mut animator in animators {
        if paused {
            animator.speed = 0.0;
            // animator.set_speed(0.0);
        } else {
            animator.speed = time.relative_speed() as f64;
        }
    }
}

// pub fn update_asset_animator_speed<T: Asset>(
//     animators: Query<&mut AssetAnimator<T>, With<PhysicalTimeAnimator>>,
//     current_state: Res<State<Pause>>,
//     time: Res<Time<Physics>>,
// ) {
//     let paused = current_state.get().0;
//     for mut animator in animators {
//         if paused {
//             animator.set_speed(0.0);
//         } else {
//             animator.set_speed(time.relative_speed());
//         }
//     }
// }

// #[cfg(feature = "dev")]
pub fn configure_gizmos(mut gizmo_config: ResMut<GizmoConfigStore>) {
    gizmo_config
        .config_mut::<DefaultGizmoConfigGroup>()
        .0
        .line
        .width = 8.0;
}
