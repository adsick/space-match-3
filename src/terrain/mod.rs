//!
//! This handles placement of the orbs in the level according to an underlying simplex noise function.
//! To ensure the terrain is infinite, the terrain made up of square chunks `CHUNK_SIZE` units wide,
//! that are populated on the fly as the player moves around.
//!

use std::{collections::VecDeque, time::Instant};

use avian2d::parry::utils::hashmap::HashMap;
use bevy::{
    color::palettes::css::{GRAY, WHEAT, WHITE},
    ecs::{spawn::SpawnIter, world::SpawnBatchIter},
    prelude::*,
    render::mesh::CircleMeshBuilder,
};
use noiz::{Noise, SampleableFor, prelude::common_noise::Simplex};

use crate::player::Player;

pub fn plugin(app: &mut App) {
    app.insert_resource(TerrainGenerator::default())
        .insert_resource(PopulatedChunks::default())
        .add_observer(populate_chunk)
        .add_systems(FixedUpdate, (trigger_chunk_population, unload_far_chunks));
}

const CHUNK_SIZE: f32 = 8.0; // TODO: Increase this
/// Number of orbs per m²
const MAX_CLOUD_DENSITY: f32 = 3.0;

// Chunks that have already been spawned.
#[derive(Default, Resource)]
pub struct PopulatedChunks(HashMap<IVec2, Entity>);

#[derive(Resource, Default)]
pub struct TerrainGenerator {
    noise: Noise<Simplex>,
    // queue: VecDeque<IVec2>
}

impl TerrainGenerator {
    pub fn sample(&self, p: Vec2) -> f32 {
        self.noise.sample(p / 100.0)
    }
}

/// Populate new chunks
fn trigger_chunk_population(
    mut cmds: Commands,
    populated: Res<PopulatedChunks>,
    q_player: Single<&Transform, With<Player>>,
) {
    let player_tr = q_player.into_inner();

    let player_tr_2d = player_tr.translation.truncate();

    let player_chunk_coord = (player_tr_2d / CHUNK_SIZE).floor().as_ivec2();

    let mut closest = None;
    let mut min_d = i32::MAX;

    let r = 7;

    // todo: more efficient traversal and quit on first match
    for y in -r..=r {
        for x in -r..=r {
            let chunk_coords = player_chunk_coord + IVec2::new(x, y);

            if !populated.0.contains_key(&chunk_coords) {
                let d = IVec2::new(x, y).length_squared();

                if d < min_d {
                    closest = Some(chunk_coords);
                    min_d = d;
                }
            }
        }
    }

    if let Some(chunk_coords) = closest {
        let pos = chunk_coords.as_vec2().extend(0.0) * CHUNK_SIZE;
        let chunk_entity = cmds
            .spawn((
                Transform::from_translation(pos),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        cmds.trigger_targets(PopulateChunk(chunk_coords), chunk_entity);
    }
}

#[derive(Debug, Event)]
pub struct PopulateChunk(IVec2);

/// Observer that populates a chunk with orb clouds.
/// The chunk is first subdivided into `CHUNK_SUBDIV` parts along each axis,
/// then each cell may spawn an orb depending on randomness and underlying terrain parameters.
fn populate_chunk(
    trigger: Trigger<PopulateChunk>,
    mut cmds: Commands,
    terrain: Res<TerrainGenerator>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut populated: ResMut<PopulatedChunks>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Calculate how many subdivisions along each axis is required to get the desired maximum cloud density.
    const CHUNK_SUBDIV: usize = ((MAX_CLOUD_DENSITY * CHUNK_SIZE * CHUNK_SIZE) as usize).isqrt();

    // let inst = Instant::now(); // ! panic on wasm

    let mut entities = vec![];

    for y in 0..CHUNK_SUBDIV {
        for x in 0..CHUNK_SUBDIV {
            let cell_pos = (Vec2::new(x as f32, y as f32) / (CHUNK_SUBDIV as f32)) * CHUNK_SIZE;
            let r = terrain.sample(trigger.0.as_vec2() * CHUNK_SIZE + cell_pos);
            // let r = 0.5;

            if r < 0.1 {
                continue;
            }

            // The actual orb position is slightly offset to avoid a grid-like look
            // TODO: improve random generation perf
            let pos = cell_pos
                + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                    / CHUNK_SUBDIV as f32;

            // let pos = cell_pos;

            let resolution = 3 + (6.0 * r) as u32;
            // TODO: Spawn an orb here
            // cmds.entity(trigger.target()).with_child(bundle)
            entities.push((
                Mesh3d(meshes.add(CircleMeshBuilder::new(0.44 * r, resolution).build())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WHEAT.with_alpha(4.0 / (1.0 + r * 20.0)).into(),
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    emissive: (WHITE * 4.0).into(),
                    ..Default::default()
                })),
                Transform::from_translation(pos.extend((rand::random::<f32>() - 0.5) * 20.0 * r)),
            ));
        }
    }

    debug!("new orbs: {}", entities.len());
    cmds.entity(trigger.target()).with_children(|parent| {
        for b in entities {
            parent.spawn(b);
        }
    });
    // cmds.entity(trigger.target())
    //     .insert(Children::spawn(SpawnIter(entities.into_iter())));

    // let t = inst.elapsed();

    // debug!("chunk generation took {t:.2?}");

    populated.0.insert(trigger.0, trigger.target());
}

fn unload_far_chunks(
    mut cmds: Commands,
    mut populated: ResMut<PopulatedChunks>,
    player: Single<&Transform, With<Player>>,
) {
    let player_chunk_coord = (player.translation.truncate() / CHUNK_SIZE)
        .floor()
        .as_ivec2();
    for (chunk_coords, chunk_entity) in populated.0.clone().iter() {
        // need to figure out this const
        if player_chunk_coord.distance_squared(*chunk_coords) > 64 {
            populated.0.remove(chunk_coords);
            cmds.entity(*chunk_entity).despawn();
        }
    }
}
