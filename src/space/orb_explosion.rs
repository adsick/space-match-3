use std::{collections::VecDeque, time::Duration};

use bevy::{
    app::{App, Update},
    math::Vec2,
    prelude::*,
    time::Time,
};
use bevy_spatial::{SpatialAccess, kdtree::KDTree2};

use super::{GasGenerator, GasOrb};

pub fn plugin(app: &mut App) {
    app.add_event::<OrbExplosion>()
        .add_systems(Update, burn_explosion);
    // app.add_observer(on_explosion_spawned);
}

#[derive(Event)]
pub struct OrbExplosion {
    pub pos: Vec2,
}

struct OrbExplosionCell {
    pos: Vec2,
    time: u32,
    distance: u32,
}

fn burn_explosion(
    mut events: EventReader<OrbExplosion>,
    mut commands: Commands,
    // gas_generator: Res<GasGenerator>,
    tree: Res<KDTree2<GasOrb>>,

    time: Res<Time>,
    mut explosion_queue: Local<Option<VecDeque<OrbExplosionCell>>>,
) {
    const CELL_SIZE: f32 = 15.;
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
        if explosion.distance > 10 {
            return false;
        }

        if curr_time > explosion.time + 50 * explosion.distance {
            let mut burnt_orbs = false;
            for (_, entity) in tree.within_distance(explosion.pos, CELL_SIZE / 2.) {
                if let Some(e) = entity {
                    commands.entity(e).try_despawn();
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
