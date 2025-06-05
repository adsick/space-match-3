//!
//! This handles placement of the orbs in the level according to an underlying simplex noise function.
//! To ensure the space is infinite it's made up of square chunks `CHUNK_SIZE` units wide,
//! that are populated on the fly as the player moves around.
//!

use avian2d::parry::utils::hashmap::HashMap;
use bevy::prelude::*;
use noiz::{Noise, SampleableFor, prelude::common_noise::Perlin, rng::NoiseRng};

use crate::{gas::GasOrb, player::Player};

pub fn plugin(app: &mut App) {
    app.insert_resource(GasGenerator {
        noise: Noise {
            noise: Perlin::default(),
            seed: NoiseRng(rand::random()), // it was 0 by default so every time you loaded the game it was the same
            frequency: 0.004,
        },
    })
    .insert_resource(PopulatedChunks::default())
    .add_observer(populate_chunk)
    .add_systems(
        FixedUpdate,
        (trigger_chunk_population, unload_far_chunks).chain(),
    );
}

pub const CHUNK_SIZE: f32 = 64.0; // TODO: Increase this
/// Number of orbs per m²
pub const MAX_CLOUD_DENSITY: f32 = 0.03;
pub const RENDER_DISTANCE: i32 = 8;
pub const THRESHOLD: f32 = 0.1;

pub const MIN_ORB_SIZE: f32 = 0.4;
pub const ORB_SCALE: f32 = 4.0;
pub const CLOUD_Z_SCALE: f32 = 90.0;

// Chunks that have already been spawned.
#[derive(Default, Resource)]
pub struct PopulatedChunks(HashMap<IVec2, Entity>);

#[derive(Resource, Default)]
pub struct GasGenerator {
    noise: Noise<Perlin>,
}

impl GasGenerator {
    pub fn sample(&self, p: Vec2) -> f32 {
        let offset: Vec2 = Vec2::new(
            self.noise.sample(p * 2.0 + 100.0),
            self.noise.sample(p * 2.0 + 200.0),
        ) * 100.0;
        self.noise.sample(p + offset)
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

    let r = RENDER_DISTANCE;

    // todo: more efficient traversal and quit on first match
    for y in -r..=r {
        for x in -r..=r {
            let chunk_coords = player_chunk_coord + IVec2::new(x, y);

            if !populated.0.contains_key(&chunk_coords) {
                // let pos = chunk_coords.as_vec2().extend(0.0) * CHUNK_SIZE;
                // let chunk_entity = cmds
                //     .spawn((
                //         Transform::from_translation(pos),
                //         InheritedVisibility::VISIBLE,
                //     ))
                //     .id();
                // cmds.trigger_targets(PopulateChunk(chunk_coords), chunk_entity);

                let d = IVec2::new(x, y).length_squared();

                if d < min_d {
                    closest = Some(chunk_coords);
                    min_d = d;
                }
            }
        }
    }

    if let Some(chunk_coords) = closest {
        // let pos = chunk_coords.as_vec2().extend(0.0) * CHUNK_SIZE;
        let chunk_entity = cmds
            .spawn((Transform::default(), InheritedVisibility::VISIBLE))
            .id();
        cmds.trigger_targets(PopulateChunk(chunk_coords), chunk_entity);
    }
}

#[derive(Debug, Event)]
pub struct PopulateChunk(IVec2);

/// Observer that populates a chunk with orb clouds.
/// The chunk is first subdivided into `CHUNK_SUBDIV` parts along each axis,
/// then each cell may spawn an orb depending on randomness and underlying space parameters.
fn populate_chunk(
    trigger: Trigger<PopulateChunk>,
    mut cmds: Commands,
    gas: Res<GasGenerator>,
    mut populated: ResMut<PopulatedChunks>,
) {
    // Calculate how many subdivisions along each axis is required to get the desired maximum cloud density.
    const CHUNK_SUBDIV: usize = ((MAX_CLOUD_DENSITY * CHUNK_SIZE * CHUNK_SIZE) as usize).isqrt();

    // let inst = Instant::now(); // ! panic on wasm

    for y in 0..CHUNK_SUBDIV {
        for x in 0..CHUNK_SUBDIV {
            let cell_pos = trigger.0.as_vec2() * CHUNK_SIZE
                + (Vec2::new(x as f32, y as f32) / (CHUNK_SUBDIV as f32)) * CHUNK_SIZE;
            let r = gas.sample(cell_pos);
            // let r = 0.5;

            if r < THRESHOLD {
                continue;
            }

            // The actual orb position is slightly offset to avoid a grid-like look
            // TODO: improve random generation perf
            let pos = cell_pos
                + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                    / CHUNK_SUBDIV as f32;

            cmds.spawn((
                GasOrb,
                Transform::from_translation(
                    pos.extend((rand::random::<f32>() - 0.5) * CLOUD_Z_SCALE * r),
                ) // todo: we can vary that 0.5 with another noise for more depth effect
                .with_scale(Vec3::splat(MIN_ORB_SIZE + ORB_SCALE * r)),
                ChildOf(trigger.target()),
            ));
        }
    }

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
        if player_chunk_coord.distance_squared(*chunk_coords)
            > RENDER_DISTANCE * RENDER_DISTANCE * 6 / 5
        {
            populated.0.remove(chunk_coords);
            cmds.entity(*chunk_entity).despawn();
        }
    }
}
