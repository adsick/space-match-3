use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, OnCollisionStart, RigidBody,
};
use bevy::{
    app::App,
    asset::Assets,
    color::palettes::css::GREEN,
    math::{Vec2, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        Bundle, ChildOf, Commands, Component, Entity, Mesh, Mesh3d, Observer, OnAdd, Query, ResMut,
        Sphere, Transform, Trigger,
    },
};


const MIN_METEORITE_SIZE: f32 = 10.0;
const METEORITE_SIZE_VARIATION: f32 = 15.0;
const METEORITE_CLOUD_Z_SCALE: f32 = 10.0;

pub fn plugin(app: &mut App) {
    app.add_observer(on_add_meteorite)
        .add_observer(on_add_ship_meteorite_collider);
}

#[derive(Component)]
pub struct Meteorite {
    pub pos: Vec2,
}

#[derive(Component)]
pub struct ShipMeteoriteCollider {}

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

fn on_add_meteorite(
    trigger: Trigger<OnAdd, Meteorite>,
    mut commands: Commands,
    meteorites: Query<&Meteorite>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut meteorite_materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    let Ok(meteorite) = meteorites.get(entity) else {
        return;
    };

    let r = rand::random::<f32>();
    let meteorite_size = MIN_METEORITE_SIZE + METEORITE_SIZE_VARIATION * r;

    commands.entity(entity).insert((
        RigidBody::Kinematic,
        // CollisionLayers::new(
        //     GameCollisionLayers::Meteorites,
        //     GameCollisionLayers::Meteorites,
        // ),
        Collider::circle(meteorite_size),
        Transform::from_translation(
            meteorite
                .pos
                .extend((rand::random::<f32>() - 0.5) * METEORITE_CLOUD_Z_SCALE),
        ),
        // .with_scale(Vec3::splat(meteorite_size)),
        Mesh3d(meshes.add(Sphere::new(meteorite_size))),
        MeshMaterial3d(meteorite_materials.add(StandardMaterial {
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

fn on_add_ship_meteorite_collider(
    trigger: Trigger<OnAdd, ShipMeteoriteCollider>,
    mut commands: Commands,
) {
    let entity = trigger.target();

    commands
        .entity(entity)
        .insert((CollisionEventsEnabled,))
        .observe(
            |trigger: Trigger<OnCollisionStart>,
             mut commands: Commands,
             meteorites: Query<&Meteorite>| {
                // let meteorite = meteorites.get(trigger.collider);

                let Ok(meteorite) = meteorites.get(trigger.collider) else {
                    return;
                };

                println!("collision");

                commands.entity(trigger.collider).despawn();

                // let pressure_plate = trigger.target();
                // let other_entity = trigger.collider;
                // if player_query.contains(other_entity) {
                //     println!("Player {other_entity} stepped on pressure plate {pressure_plate}");
                // }
            },
        );
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

fn on_collision() {}
