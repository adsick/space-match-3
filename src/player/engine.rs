use std::collections::VecDeque;

use avian2d::prelude::{LinearVelocity, Physics};
use bevy::{
    color::{
        ColorToComponents,
        palettes::css::{PURPLE, RED},
    },
    math::{UVec4, Vec2, Vec3Swizzles, Vec4, VectorSpace},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Changed, Commands, Component, GlobalTransform, Local, Mesh, Mesh3d, OnAdd,
        Query, Rectangle, Res, ResMut, Single, Trigger, With, *,
    },
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{
    PausableSystems,
    player::{Player, movement::CurrentGas},
    screens::Screen,
};

const FIRE_SHADER_PATH: &str = "shaders/rocket_fire.wgsl";
const NOF_PARTICLES: usize = 20;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<EngineFire>()
        .add_observer(on_add_fire)
        .add_plugins((MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, FireMaterialExtension>,
        >::default(),))
        .add_systems(
            Update,
            check_fire_params_change
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        )
        .add_systems(
            Update,
            (update_engine_power, update_shader_params)
                .chain()
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        );
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
            dir: Vec4::X,
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

#[derive(Component, Reflect)]
pub struct EngineFire {
    /// Use 0.5. Other values look cringe.
    pub power: f32,
    pub color: Vec4,
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
        Mesh3d(meshes.add(Rectangle::from_length(2000.))),
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

fn update_engine_power(
    mut fire_query: Query<(&mut EngineFire, &ChildOf)>,
    ship_query: Query<&CurrentGas>,
) {
    for (mut fire_params, child_of) in &mut fire_query {
        let Ok(current_gas) = ship_query.get(child_of.parent()) else {
            return;
        };
        fire_params.color = (Srgba::new(252. / 255., 10. / 255., 113. / 255., 1.0) * 2.0)
            .lerp(PURPLE * 2.0, current_gas.0)
            .to_vec4();
    }
}

fn update_shader_params(
    ship: Single<&LinearVelocity, With<Player>>,
    fire: Single<(&GlobalTransform, &EngineFire)>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    fire_material: Single<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        With<EngineFire>,
    >,

    mut particles_queue: Local<VecDeque<(Vec2, Vec2)>>,
    mut last_particle_spawned: Local<u128>,

    time: Res<Time<Physics>>,
) {
    let Some(fire_material) = materials.get_mut(*fire_material) else {
        return;
    };

    let (fire_tr, fire_params) = *fire;
    let ship_velocity = ship.into_inner();

    let flame_dir = fire_tr.down();
    let curr_time = time.elapsed().as_millis();

    let td = time.delta_secs();

    let origin = fire_tr.translation().xy();

    for (pos, vel) in &mut particles_queue {
        *pos += *vel * td;
        *vel *= 0.9;
    }

    // there is some kind of inconsistency going on which causes the start of the fire to flicker on turns.
    // idk why is that happening.

    if curr_time - *last_particle_spawned > 0 {
        let new_particle = (origin, flame_dir.xy() * 100.0); // this particle velocity can be turned into a resource/component
        particles_queue.push_front(new_particle);

        if particles_queue.len() >= NOF_PARTICLES {
            particles_queue.pop_back();
        }
        *last_particle_spawned = curr_time;
    }

    for (i, (pos, _)) in particles_queue.iter().enumerate() {
        fire_material.extension.particles[i] = pos.extend(0.0).extend(0.0);
    }

    fire_material.extension.nof_particles = UVec4::splat(particles_queue.len() as u32);

    fire_material.extension.dir = -flame_dir.extend(0.0);

    fire_material.extension.center = origin.extend(0.0).extend(0.0);

    // let color = RED.lerp(PURPLE, fire_params.power);
    fire_material.extension.color = fire_params.color;

    fire_material.extension.power = Vec4::splat(fire_params.power);
}
