use std::{collections::VecDeque, time::Duration};

use bevy::{
    app::{App, Update},
    math::Vec2,
    prelude::*,
    time::Time,
};
use bevy_spatial::{SpatialAccess, kdtree::KDTree2};

use crate::gas::{BurningGasOrb, assets::OrbAssets};

use super::{GasGenerator, GasOrb};

pub fn plugin(app: &mut App) {
    app.add_event::<OrbExplosion>()
        .add_systems(Update, (propagate_explosion, update_burning_orbs));
    // app.add_observer(on_explosion_spawned);
}

const BURN_TIME: u32 = 670;

#[derive(Event)]
pub struct OrbExplosion {
    pub pos: Vec2,
}

struct OrbExplosionCell {
    pos: Vec2,
    time: u32,
    distance: u32,
}

fn propagate_explosion(
    mut events: EventReader<OrbExplosion>,
    mut commands: Commands,
    // gas_generator: Res<GasGenerator>,
    tree: Res<KDTree2<GasOrb>>,

    time: Res<Time>,
    mut explosion_queue: Local<Option<VecDeque<OrbExplosionCell>>>,
) {
    const CELL_SIZE: f32 = 14.;
    let explosion_queue = explosion_queue.get_or_insert(VecDeque::new());
    let curr_time = time.elapsed().as_millis() as u32;

    for event in events.read() {
        explosion_queue.push_back(OrbExplosionCell {
            pos: event.pos,
            time: curr_time,
            distance: 0,
        });
    }

    debug!("explosions: {}", explosion_queue.len());

    let mut new_explosions = Vec::<OrbExplosionCell>::new();

    explosion_queue.retain(|explosion| {
        if explosion.distance > 8 {
            return false;
        }

        // condition of burn propagation. basically the older the explosion is the longer it takes to propagate
        if curr_time > explosion.time + 10 + 30 * explosion.distance {
            let mut burnt_orbs = false;
            for (_, entity) in tree.within_distance(explosion.pos, CELL_SIZE / 2.) {
                if let Some(e) = entity {
                    commands
                        .entity(e)
                        .try_remove::<GasOrb>()
                        .try_insert(BurningGasOrb(curr_time));
                    burnt_orbs = true;
                }
            }

            if !burnt_orbs {
                // debug!("no orbs burnt");
                return false;
            }

            let neis = [
                explosion.pos + Vec2::Y * CELL_SIZE,
                explosion.pos - Vec2::Y * CELL_SIZE,
                explosion.pos + Vec2::X * CELL_SIZE,
                explosion.pos - Vec2::X * CELL_SIZE,
            ];

            for nei in neis {
                new_explosions.push(OrbExplosionCell {
                    pos: nei,
                    time: curr_time,
                    distance: explosion.distance + 1,
                });
            }
            false // here I delete the explosion
        } else {
            true
        }
    });

    for explosion in new_explosions {
        explosion_queue.push_back(explosion);
    }
}

fn update_burning_orbs(
    commands: Commands,
    mut orb_q: Query<(
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
        &BurningGasOrb,
    )>,
    orb_assets: Res<OrbAssets>,
    time: Res<Time>,
) {
    let ct = time.elapsed().as_millis() as u32;

    orb_q.par_iter_mut().for_each(|(mut tr, mut mat, time)| {
        let dt = ct - time.0;

        if dt > 2 * BURN_TIME {
            mat.0 = orb_assets.orb_materials[3].clone();
        } else if dt > BURN_TIME {
            mat.0 = orb_assets.orb_materials[2].clone();
            tr.scale *= 0.996;
        } else {
            mat.0 = orb_assets.orb_materials[1].clone();
            tr.scale *= 1.011;
            tr.scale = tr.scale.min(Vec3::splat(100.0))
        }
    });
}

// reference
fn animate_materials(
    material_handles: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            if let Color::Hsla(ref mut hsla) = material.base_color {
                *hsla = hsla.rotate_hue(time.delta_secs() * 100.0);
            }
        }
    }
}
