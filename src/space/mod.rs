//!
//! This handles placement of the orbs in the level according to an underlying simplex noise function.
//! To ensure the space is infinite it's made up of square chunks `CHUNK_SIZE` units wide,
//! that are populated on the fly as the player moves around.
//!

use std::cmp;

use avian2d::parry::utils::hashmap::HashMap;
use bevy::prelude::*;
use noiz::{Noise, SampleableFor, prelude::common_noise::Perlin, rng::NoiseRng};

use crate::{asteroids::Asteroid, gas::GasOrb, player::Player};

pub mod orb_explosion;

pub fn plugin(app: &mut App) {
    app.add_plugins(orb_explosion::plugin)
        .insert_resource(GasGenerator {
            noise: Noise {
                noise: Perlin::default(),
                seed: NoiseRng(rand::random()), // it was 0 by default so every time you loaded the game it was the same
                frequency: 0.004,
            },
        })
        .insert_resource(PopulatedChunks::default())
        .add_observer(populate_chunk)
        .add_systems(
            Update,
            (trigger_chunk_population, unload_far_chunks).chain(),
        );
}

pub const CHUNK_SIZE: f32 = 64.0; // TODO: Increase this
/// Number of orbs per m²
pub const MAX_CLOUD_DENSITY: f32 = 0.03;
pub const RENDER_DISTANCE: i32 = 12;
pub const ORB_THRESHOLD: f32 = 0.1;
pub const METEORITE_THRESHOLD: f32 = 0.99;

const MIN_ASTEROID_SIZE: f32 = 10.0;
const ASTEROID_SIZE_VARIATION: f32 = 15.0;
const ASTEROID_CLOUD_Z_SCALE: f32 = 10.0;

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

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl GasGenerator {
    pub fn sample(&self, p: Vec2) -> f32 {
        // // No orbs should be spawned near the start (0, 0) in order to free up space for the intro
        // // scene.
        // let dist_sq = p.length_squared();
        // const START_RADIUS: f32 = 500.;
        // const START_RADIUS_SQ: f32 = START_RADIUS * START_RADIUS;
        // let start_mask = smoothstep(0., START_RADIUS_SQ, dist_sq);

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

fn meteorite_distribution(r: f32) -> f32 {
    let a = smoothstep(0.1, 0.2, r);
    let b = smoothstep(0.3, 0.2, r);

    a.min(b)
}

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

            if r > ORB_THRESHOLD {
                // The actual orb position is slightly offset to avoid a grid-like look
                // TODO: improve random generation perf
                let pos = cell_pos
                    + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                        / CHUNK_SUBDIV as f32;

                cmds.spawn((
                    GasOrb(r),
                    Transform::from_translation(
                        pos.extend((rand::random::<f32>() - 0.5) * CLOUD_Z_SCALE * r),
                    ) // todo: we can vary that 0.5 with another noise for more depth effect
                    .with_scale(Vec3::splat(MIN_ORB_SIZE + ORB_SCALE * r)),
                    ChildOf(trigger.target()),
                ));
            }

            let mut meteorite_r = meteorite_distribution(r);
            meteorite_r -= rand::random::<f32>() * 0.5;
            if meteorite_r > METEORITE_THRESHOLD {
                let pos = cell_pos
                    + Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * CHUNK_SIZE
                        / CHUNK_SUBDIV as f32;

                let r = rand::random::<f32>();
                let asteroid_size = MIN_ASTEROID_SIZE + ASTEROID_SIZE_VARIATION * r;
                cmds.spawn((
                    Asteroid {
                        pos: pos.extend((rand::random::<f32>() - 0.5) * ASTEROID_CLOUD_Z_SCALE),
                        radius: asteroid_size,
                    },
                    ChildOf(trigger.target()),
                ));
            }
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
