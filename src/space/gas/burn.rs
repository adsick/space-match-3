use std::collections::VecDeque;

use avian2d::prelude::Physics;
use bevy::{
    app::{App, Update},
    math::Vec2,
    prelude::*,
    time::Time,
};
use bevy_spatial::{SpatialAccess, kdtree::KDTree2};

use crate::{
    PausableSystems,
    red_gas::{RedGasOrb, RedOrbExplosionEvent},
    screens::Screen,
    space::gas::{BurningGasOrb, assets::OrbAssets, ignite_gas},
};

use super::GasOrb;

pub fn plugin(app: &mut App) {
    app.add_event::<OrbExplosion>()
        .configure_sets(
            Update,
            UpdateGasSet
                .after(ignite_gas)
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        )
        .add_systems(
            Update,
            (propagate_explosion, update_burning_orbs).in_set(UpdateGasSet),
        );
}

const BURN_TIME: u32 = 670;
const CELL_SIZE: f32 = 16.;
const MAX_COUNT: u32 = 1000;
const LIFETIME: u32 = 30;
const SLOWDOWN: u32 = 10;
const BASE_DELAY: u32 = 30;

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub struct UpdateGasSet;

#[derive(Event)]
pub struct OrbExplosion {
    pub pos: Vec2,
}

pub struct OrbExplosionCell {
    pos: Vec2,
    time: u32,
    life: u32,
}

pub fn propagate_explosion(
    mut events: EventReader<OrbExplosion>,
    mut commands: Commands,
    orb_tree: Res<KDTree2<GasOrb>>,
    red_orb_tree: Res<KDTree2<RedGasOrb>>,

    mut red_orb_explosion_events: EventWriter<RedOrbExplosionEvent>,

    time: Res<Time<Physics>>,
    mut explosion_queue: Local<VecDeque<OrbExplosionCell>>,
) {
    let curr_time = time.elapsed().as_millis() as u32;

    for event in events.read() {
        explosion_queue.push_back(OrbExplosionCell {
            pos: event.pos,
            time: curr_time,
            life: LIFETIME,
        });
    }

    debug!("explosions: {}", explosion_queue.len());

    let mut new_explosions = Vec::<OrbExplosionCell>::new();

    let mut i = 0;
    let variation = [0.3, 0.7, 0.4, 0.6, 0.8]; // in total 2.8, 0.5 avg

    explosion_queue.retain(|explosion| {
        if explosion.life == 0 {
            return false;
        }

        let size = CELL_SIZE * variation[i % variation.len()];
        i += 1;

        // condition of burn propagation. basically the older the explosion is the longer it takes to propagate
        if curr_time > explosion.time + BASE_DELAY + SLOWDOWN * (LIFETIME - explosion.life) {
            let mut burnt_orbs = false;
            for (_, entity) in orb_tree.within_distance(explosion.pos, size / 2.0) {
                if let Some(e) = entity {
                    commands
                        .entity(e)
                        .try_remove::<GasOrb>()
                        .try_insert(BurningGasOrb(curr_time));
                    burnt_orbs = true;
                }
            }
            for (_, entity) in red_orb_tree.within_distance(explosion.pos, size / 2.0) {
                if let Some(e) = entity {
                    red_orb_explosion_events.write(RedOrbExplosionEvent { entity: e });
                    // commands
                    //     .entity(e)
                    //     .try_remove::<GasOrb>()
                    //     .try_insert(BurningGasOrb(curr_time));
                    // burnt_orbs = true;
                }
            }

            if !burnt_orbs {
                return false;
            }

            let neis = [
                explosion.pos + Vec2::Y * size,
                explosion.pos - Vec2::Y * size,
                explosion.pos + Vec2::X * size,
                explosion.pos - Vec2::X * size,
            ];

            for nei in neis {
                let life = explosion.life - 1;

                if life > 0 {
                    new_explosions.push(OrbExplosionCell {
                        pos: nei,
                        time: curr_time,
                        life,
                    });
                }
            }
            false
        } else {
            true
        }
    });

    let new_len = new_explosions.len();
    let free_space = MAX_COUNT.saturating_sub(explosion_queue.len() as u32);

    for explosion in new_explosions {
        i += 1;

        if (variation[i % variation.len()] * new_len as f32) < free_space as f32 {
            explosion_queue.push_back(explosion);
        }
    }
}

fn update_burning_orbs(
    mut orb_q: Query<(
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
        &BurningGasOrb,
    )>,
    orb_assets: Res<OrbAssets>,
    time: Res<Time<Physics>>,
) {
    let ct = time.elapsed().as_millis() as u32;
    let delta = time.delta_secs();

    orb_q.par_iter_mut().for_each(|(mut tr, mut mat, time)| {
        let dt = ct - time.0;

        if dt > 2 * BURN_TIME {
            mat.0 = orb_assets.orb_materials[3].clone();
        } else if dt > BURN_TIME {
            mat.0 = orb_assets.orb_materials[2].clone();
            tr.scale = tr.scale.lerp(tr.scale * 0.996, 60.0 * delta);
        } else {
            mat.0 = orb_assets.orb_materials[1].clone();
            tr.scale = tr
                .scale
                .lerp(tr.scale * 1.011, 60.0 * delta)
                .min(Vec3::splat(100.0));
        }
    });
}
