//!
//! This handles placement of the orbs in the level according to an underlying simplex noise function.
//! To ensure the terrain is infinite, the terrain made up of square chunks `CHUNK_SIZE` units wide,
//! that are populated on the fly as the player moves around.
//!

use std::collections::HashSet;

use bevy::{color::palettes::css::WHEAT, prelude::*, render::mesh::CircleMeshBuilder};
use noiz::{Noise, SampleableFor, prelude::common_noise::Simplex};

use crate::player::Player;

pub fn plugin(app: &mut App) {
    app.insert_resource(TerrainGenerator(default()))
        .insert_resource(PopulatedChunks::default())
        .add_observer(populate_chunk)
        .add_systems(Update, trigger_chunk_population);
}

const CHUNK_SIZE: f32 = 7.0; // TODO: Increase this
/// Number of orbs per m²
const MAX_CLOUD_DENSITY: f32 = 2.0;

// Chunks that have already been spawned.
#[derive(Default, Resource)]
pub struct PopulatedChunks(HashSet<IVec2>);

#[derive(Resource)]
struct TerrainGenerator(Noise<Simplex>);

impl TerrainGenerator {
    pub fn orb_probability(&self, p: Vec2) -> f32 {
        self.0.sample(p / 10.0)
    }
}

/// Populate new chunks
fn trigger_chunk_population(
    mut cmds: Commands,
    mut populated: ResMut<PopulatedChunks>,
    q_player: Query<&Transform, With<Player>>,
) {
    let Ok(player_tr) = q_player.single() else {
        return;
    };

    let player_chunk_coord = (player_tr.translation.truncate() / CHUNK_SIZE)
        .floor()
        .as_ivec2();
    for y in [-1, 0, 1] {
        for x in [-1, 0, 1] {
            let chunk_coords = player_chunk_coord + IVec2::new(x, y);
            if populated.0.insert(chunk_coords) {
                cmds.trigger(PopulateChunk(chunk_coords));
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
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Calculate how many subdivisions along each axis is required to get the desired maximum cloud density.
    const CHUNK_SUBDIV: usize = ((MAX_CLOUD_DENSITY * CHUNK_SIZE * CHUNK_SIZE) as usize).isqrt();

    for y in 0..CHUNK_SUBDIV {
        for x in 0..CHUNK_SUBDIV {
            let cell_pos = (trigger.0.as_vec2()
                + Vec2::new(x as f32, y as f32) / (CHUNK_SUBDIV as f32))
                * CHUNK_SIZE as f32;

            let r = terrain.orb_probability(cell_pos);

            if r < 0.16 {
                continue;
            }

            // The actual orb position is slightly offset to avoid a grid-like look
            let pos = cell_pos
                + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                    / CHUNK_SUBDIV as f32;

            // TODO: Spawn an orb here
            cmds.spawn((
                Mesh2d(meshes.add(CircleMeshBuilder::new(0.2 * r, 10).build())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(WHEAT))),
                // Sprite {
                //     custom_size: Some(Vec2::splat(CHUNK_SIZE / CHUNK_SUBDIV as f32 * 0.3)),
                //     ..default()
                // },
                Transform::from_translation(pos.extend(0.0)),
            ));
        }
    }
}
