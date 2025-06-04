use std::collections::VecDeque;

use bevy::{
    app::{App, Update},
    asset::{Asset, Assets},
    color::{
        ColorToComponents,
        palettes::css::{PURPLE, RED},
    },
    math::{UVec4, Vec2, Vec3, Vec3Swizzles, Vec4, VectorSpace},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Changed, Commands, Component, GlobalTransform, Local, Mesh, Mesh3d, OnAdd,
        Query, Rectangle, Res, ResMut, Transform, Trigger, With,
    },
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderRef},
    time::Time,
};

const FIRE_SHADER_PATH: &str = "shaders/rocket_fire.wgsl";
const NOF_PARTICLES: usize = 20;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_add_fire)
        .add_plugins((MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, FireMaterialExtension>,
        >::default(),))
        .add_systems(Update, check_fire_params_change)
        .add_systems(Update, update_shader_params);
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct FireMaterialExtension {
    #[uniform(100)]
    color: Vec4,
    #[uniform(101)]
    center: Vec4,
    #[uniform(102)]
    nof_particles: UVec4,

    #[uniform(103)]
    particles: [Vec4; 32],

    #[uniform(104)]
    dir: Vec4,

    #[uniform(105)]
    power: Vec4,
}

impl Default for FireMaterialExtension {
    fn default() -> Self {
        Self {
            color: Vec4::new(0., 0.0, 0.0, 0.0),
            center: Vec4::new(0.0, 0.0, 0.0, 0.0),
            nof_particles: UVec4::new(NOF_PARTICLES as u32, 0, 0, 0),
            particles: [Vec4::ZERO; 32],
            dir: Vec3::Y.extend(0.0),
            power: Vec4::new(1.0, 0., 0., 0.),
        }
    }
}

impl MaterialExtension for FireMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        FIRE_SHADER_PATH.into()
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }
}

#[derive(Component)]
pub struct EngineFire {
    pub power: f32,
}

fn on_add_fire(
    trigger: Trigger<OnAdd, EngineFire>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
    >,
) {
    let entity = trigger.target();

    commands.entity(entity).insert((
        // Transform::from_translation(-Vec3::Y * 0.),
        Mesh3d(meshes.add(Rectangle::from_length(200.))),
        MeshMaterial3d(explosion_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
            extension: FireMaterialExtension::default(),
        })),
    ));
}

fn check_fire_params_change(
    fire: Query<
        (
            &EngineFire,
            &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        ),
        Changed<EngineFire>,
    >,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
) {
    for (fire, fire_material) in fire {
        let Some(fire_material) = materials.get_mut(fire_material) else {
            continue;
        };

        let color = RED.to_vec4().lerp(PURPLE.to_vec4(), fire.power);
        fire_material.extension.color = color;
        fire_material.extension.power = Vec4::splat(fire.power);
    }
}

fn update_shader_params(
    ship_transforms: Query<(&GlobalTransform, &EngineFire)>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    fire_material: Query<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        With<EngineFire>,
    >,

    mut particles_queue: Local<VecDeque<(Vec2, Vec2)>>,
    mut last_particle_spawned: Local<u128>,
    time: Res<Time>,
) {
    for (ship_transform, fire_params) in ship_transforms {
        for fire_material in fire_material {
            let Some(fire_material) = materials.get_mut(fire_material) else {
                return;
            };

            let flame_dir = ship_transform.rotation().mul_vec3(-Vec3::Y);
            let curr_time = time.elapsed().as_millis();
            if curr_time - *last_particle_spawned > 30 {
                particles_queue.push_front((ship_transform.translation().xy(), flame_dir.xy()));

                if particles_queue.len() >= NOF_PARTICLES {
                    particles_queue.pop_back();
                }

                *last_particle_spawned = curr_time;
            }

            for (pos, flame_dir) in &mut particles_queue {
                *pos += *flame_dir * 0.6 * fire_params.power;
            }

            for (i, (pos, _)) in particles_queue.iter().enumerate() {
                fire_material.extension.particles[i] = pos.extend(0.0).extend(0.0);
            }

            fire_material.extension.nof_particles = UVec4::splat(particles_queue.len() as u32);

            fire_material.extension.dir = -flame_dir.extend(0.0);

            fire_material.extension.center = ship_transform.translation().extend(0.0);

            let color = RED.lerp(PURPLE, fire_params.power);
            fire_material.extension.color = color.to_vec4();

            fire_material.extension.power = Vec4::splat(fire_params.power);
        }
    }
}
