//!
//! This handles placement of the orbs in the level according to an underlying simplex noise function.
//! To ensure the terrain is infinite, the terrain made up of square chunks `CHUNK_SIZE` units wide,
//! that are populated on the fly as the player moves around.
//!

use std::time::Instant;

use avian2d::parry::utils::hashmap::HashMap;
use bevy::{
    color::palettes::css::{GRAY, WHEAT, WHITE},
    prelude::*,
    render::mesh::CircleMeshBuilder,
};
use noiz::{Noise, SampleableFor, prelude::common_noise::Simplex};

use crate::player::Player;

pub fn plugin(app: &mut App) {
    app.insert_resource(TerrainGenerator(default()))
        .insert_resource(PopulatedChunks::default())
        .add_observer(populate_chunk)
        .add_systems(Update, (trigger_chunk_population, unload_far_chunks));
}

const CHUNK_SIZE: f32 = 20.0; // TODO: Increase this
/// Number of orbs per m²
const MAX_CLOUD_DENSITY: f32 = 3.0;

// Chunks that have already been spawned.
#[derive(Default, Resource)]
pub struct PopulatedChunks(HashMap<IVec2, Entity>);

#[derive(Resource)]
pub struct TerrainGenerator(Noise<Simplex>);

impl TerrainGenerator {
    pub fn orb_probability(&self, p: Vec2) -> f32 {
        self.0.sample(p / 10.0)
    }
}

/// Populate new chunks
fn trigger_chunk_population(
    mut cmds: Commands,
    populated: Res<PopulatedChunks>,
    q_player: Single<&Transform, With<Player>>,
) {
    let player_tr = q_player.into_inner();

    let player_chunk_coord = (player_tr.translation.truncate() / CHUNK_SIZE)
        .floor()
        .as_ivec2();

    for y in [-1, 0, 1] {
        for x in [-1, 0, 1] {
            let chunk_coords = player_chunk_coord + IVec2::new(x, y);
            if !populated.0.contains_key(&chunk_coords) {

                let pos = chunk_coords.as_vec2().extend(0.0) * CHUNK_SIZE; 
                let chunk_entity = cmds
                    .spawn((
                        Transform::from_translation(
                            pos,
                        ),
                        InheritedVisibility::VISIBLE,
                    ))
                    .id();
                debug!("spawning chunk at {pos}");
                cmds.trigger_targets(PopulateChunk(chunk_coords), chunk_entity);
            }
        }
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

    let inst = Instant::now();

    for y in 0..CHUNK_SUBDIV {
        for x in 0..CHUNK_SUBDIV {
            let cell_pos = (Vec2::new(x as f32, y as f32) / (CHUNK_SUBDIV as f32)) * CHUNK_SIZE;
            let r = terrain.orb_probability(trigger.0.as_vec2() * CHUNK_SIZE + cell_pos);

            if r < 0.16 {
                continue;
            }

            // The actual orb position is slightly offset to avoid a grid-like look
            // TODO: improve random generation perf
            let pos = cell_pos
                + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                    / CHUNK_SUBDIV as f32;

            let resolution = 3 + (6.0 * r) as u32;
            // debug!("{resolution}");
            // TODO: Spawn an orb here
            cmds.entity(trigger.target()).with_child((
                Mesh3d(meshes.add(CircleMeshBuilder::new(0.2 * r, resolution).build())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WHEAT.with_alpha(0.7).into(),
                    emissive: GRAY.into(),
                    ..Default::default()
                })),
                Transform::from_translation(pos.extend((rand::random::<f32>() - 0.5) * 10.0 * r)),
            ));
        }
    }

    let t = inst.elapsed();

    debug!("chunk generation took {t:.2?}");

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
        if player_chunk_coord.distance_squared(*chunk_coords) > 2 {
            populated.0.remove(chunk_coords);
            cmds.entity(*chunk_entity).despawn();
        }
    }
}
