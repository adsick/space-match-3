use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_spatial::{
    AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode, kdtree::KDTree2,
};

use crate::{
    gas::assets::OrbAssets,
    player::{Player, movement::CurrentGas},
    space::orb_explosion::propagate_explosion,
};

pub mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        AutomaticUpdate::<GasOrb>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_secs_f32(0.3))
            .with_transform(TransformMode::GlobalTransform),
        assets::plugin,
    ))
    .add_observer(setup)
    .add_systems(Update, pickup_gas.before(propagate_explosion));
}

#[derive(Component)]
pub struct GasOrb(pub f32); // contains it's mass

#[derive(Component)]
pub struct BurningGasOrb(pub u32); // time when it started burning in ms

// #[derive(Component)]
// pub struct AttractedGasOrb {
//     by_ship: Entity,
//     // When this reaches 1.0, will get consumed by the ship
//     time: f32,
// }

fn setup(trigger: Trigger<OnAdd, GasOrb>, mut cmds: Commands, gas_assets: Res<OrbAssets>) {
    cmds.entity(trigger.target()).insert((
        Mesh3d(gas_assets.orb_mesh.clone()),
        MeshMaterial3d(gas_assets.orb_materials[0].clone()),
    ));
}

pub fn pickup_gas(
    cmds: Commands,
    // mut q_picked_up_orbs: Query<(Entity, &mut Transform, &mut AttractedGasOrb)>,
    mut q_orbs: Query<&GasOrb>,
    mut q_ship: Single<(&Transform, &mut CurrentGas)>,
    tree: Res<KDTree2<GasOrb>>,
    time: Res<Time>,
) {
    let (ship_tr, mut gas) = q_ship.into_inner();

    let backward = ship_tr.down().truncate();
    let ship_tr_2d = ship_tr.translation.truncate();

    let mut total_gas = 0.0;

    for (orb_pos, e) in tree.within_distance(ship_tr_2d, 10.0) {
        let k = orb_pos.dot(backward).clamp(0.0, 1.0);

        if let Some(e) = e {
            q_orbs.get(e).map(|g| total_gas += k * g.0).ok();
        }
    }

    debug!("{total_gas}");

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
