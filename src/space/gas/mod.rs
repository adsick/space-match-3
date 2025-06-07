use std::time::Duration;

use bevy::prelude::*;
use bevy_spatial::{
    AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode, kdtree::KDTree2,
};

use crate::{
    PausableSystems,
    player::movement::CurrentGas,
    screens::Screen,
    space::gas::{assets::OrbAssets, burn::OrbExplosion},
};

pub mod assets;
pub mod burn;

use burn::propagate_explosion;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        AutomaticUpdate::<GasOrb>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_secs_f32(0.3))
            .with_transform(TransformMode::GlobalTransform),
        assets::plugin,
        burn::plugin,
    ))
    .add_observer(orb_setup)
    .add_systems(
        Update,
        ignite_gas
            .before(propagate_explosion)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct GasOrb(pub f32); // contains it's mass

#[derive(Component)]
pub struct BurningGasOrb(pub u32); // time when it started burning in ms

// both should not be high
pub const IGNITION_OFFSET: f32 = 10.0;
pub const IGNITION_RADIUS: f32 = 13.0;

fn orb_setup(trigger: Trigger<OnAdd, GasOrb>, mut cmds: Commands, gas_assets: Res<OrbAssets>) {
    cmds.entity(trigger.target()).insert((
        Mesh3d(gas_assets.orb_mesh.clone()),
        MeshMaterial3d(gas_assets.orb_materials[0].clone()),
    ));
}

// we can accelerate right here I guess... no need we already run before thrust. maybe pipe them then?
pub fn ignite_gas(
    q_orbs: Query<&GasOrb>,
    q_ship: Single<(&Transform, &mut CurrentGas)>,
    tree: Res<KDTree2<GasOrb>>,
    mut ignite_gas_tx: EventWriter<OrbExplosion>,
) {
    let (ship_tr, mut gas) = q_ship.into_inner();

    let backward = ship_tr.down().truncate();
    let ship_tr_2d = ship_tr.translation.truncate();

    let mut total_gas = 0.0;

    let mut count = 0;
    // this code is responsible for detecting gas that is behind the ship
    for (orb_pos, e) in tree.within_distance(ship_tr_2d + IGNITION_OFFSET * backward, IGNITION_RADIUS) {
        let k = (orb_pos - ship_tr_2d)
            .normalize()
            .dot(backward)
            .clamp(0.0, 1.0);
        if let Some(e) = e {
            count += 1;
            q_orbs.get(e).map(|g| total_gas += k * g.0).ok();
        }
    }

    if total_gas > 0.0 {
        ignite_gas_tx.write(OrbExplosion {
            pos: ship_tr_2d + IGNITION_OFFSET * backward,
        });
    }

    if count > 0 {
        debug!("count: {count}, gas: {total_gas:.2}");
    }

    gas.0 = (gas.0 + total_gas).min(1.0);
}

// fn attract_gas(
//     mut commands: Commands,
//     player_query: Single<(Entity, &Position), With<Player>>,
//     tree: Res<KDTree2<GasOrb>>,
//     q_orb: Query<(), (With<GasOrb>, Without<AttractedGasOrb>)>,
// ) {
//     let (ship_entity, position) = player_query.into_inner();
//     for (_, entity) in tree.within_distance(position.0, 5.0) {
//         if let Some(e) = entity {
//             if q_orb.contains(e) {
//                 commands.entity(e).insert(AttractedGasOrb {
//                     by_ship: ship_entity,
//                     time: 0.0,
//                 });
//             }
//         }
//     }
// }
