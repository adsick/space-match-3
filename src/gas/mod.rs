use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_spatial::{
    AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode, kdtree::KDTree2,
};

use crate::{
    gas::assets::OrbAssets,
    player::{Player, movement::CurrentGas},
};

mod assets;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        AutomaticUpdate::<GasOrb>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_secs_f32(0.3))
            .with_transform(TransformMode::GlobalTransform),
        assets::plugin,
    ))
    .add_observer(setup)
    .add_systems(Update, (attract_gas, pickup_gas));
}

#[derive(Component)]
pub struct GasOrb;

#[derive(Component)]
pub struct AttractedGasOrb {
    by_ship: Entity,
    // When this reaches 1.0, will get consumed by the ship
    time: f32,
}

fn setup(trigger: Trigger<OnAdd, GasOrb>, mut cmds: Commands, gas_assets: Res<OrbAssets>) {
    cmds.entity(trigger.target()).insert((
        Mesh3d(gas_assets.orb_meshes[2].clone()),
        MeshMaterial3d(gas_assets.orb_materials[2].clone()),
    ));
}

pub fn pickup_gas(
    mut cmds: Commands,
    mut q_picked_up_orbs: Query<(Entity, &mut Transform, &mut AttractedGasOrb)>,
    mut q_ship: Query<(&Position, &mut CurrentGas)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut picked_up) in &mut q_picked_up_orbs {
        let Ok((target_pos, mut gas)) = q_ship.get_mut(picked_up.by_ship) else {
            return;
        };
        picked_up.time += time.delta_secs();
        if picked_up.time >= 0.2 {
            gas.0 = (gas.0 + 0.02).min(1.0);
            cmds.entity(entity).despawn();
        }
        transform.translation = transform
            .translation
            .truncate()
            .lerp(target_pos.0, picked_up.time)
            .extend(transform.translation.z);
    }
}

fn attract_gas(
    mut commands: Commands,
    player_query: Single<(Entity, &Position), With<Player>>,
    tree: Res<KDTree2<GasOrb>>,
    q_orb: Query<(), (With<GasOrb>, Without<AttractedGasOrb>)>,
) {
    let (ship_entity, position) = player_query.into_inner();
    for (_, entity) in tree.within_distance(position.0, 30.0) {
        if let Some(e) = entity {
            if q_orb.contains(e) {
                commands.entity(e).insert(AttractedGasOrb {
                    by_ship: ship_entity,
                    time: 0.0,
                });
            }
        }
    }
}
