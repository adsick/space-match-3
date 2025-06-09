use std::time::Duration;

use bevy::{
    app::{App, Update},
    math::{Vec2, Vec3},
    pbr::{PointLight, StandardMaterial},
    prelude::*,
};
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use bevy_tweening::{AnimationSystem, Lens, Targetable, component_animator_system};

use crate::{PausableSystems, asset_tracking::LoadResource, screens::Screen};

pub mod assets;
pub mod logic;
mod sound;

use assets::RedOrbAssets;
use logic::*;

const MAX_EXPLOSION_RADIUS: f32 = 1500.;
const EXPLOSION_DURATION_SECS: u64 = 10;
const EXPLOSION_CLEANUP_RADIUS: f32 = 3000.;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        AutomaticUpdate::<RedGasOrb>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_secs_f32(0.1))
            .with_transform(TransformMode::GlobalTransform),
        sound::plugin,
    ))
    .add_observer(on_add_explosive_gas_orb)
    .load_resource::<RedOrbAssets>()
    .insert_resource(ExplosionDamage(0.0))
    .add_event::<RedOrbExplosionEvent>()
    .add_systems(
        Update,
        (
            explode_red_orbs,
            check_explosion_interactions,
            component_animator_system::<RedOrbExplosion>.in_set(AnimationSystem::AnimationUpdate),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    )
    .add_systems(
        Update,
        (
            update_component_animator_speed::<PointLight>,
            update_component_animator_speed::<RedOrbExplosion>,
            update_component_animator_speed::<Transform>,
            // update_asset_animator_speed::<StandardMaterial>,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    #[cfg(feature = "dev")]
    app.add_systems(Startup, configure_gizmos);
}

#[derive(Resource)]
/// When the damage reaches 1.0, the player must die.
pub struct ExplosionDamage(pub f32);

#[derive(Component)]
pub struct RedGasOrb {
    pub radius: f32,
    pub pos: Vec3,
}

#[derive(Component)]
pub struct RedOrbExplosion {
    radius: f32,
    pos: Vec2,
    // The number of other orbs this one has interacted with. For optimization purposes.
    interactions: usize,
}

#[derive(Event)]
pub struct RedOrbExplosionEvent {
    pub entity: Entity,
    pub meta: u8,
}

#[derive(Component)]
pub struct PhysicalTimeAnimator;

#[derive(Debug, Copy, Clone, PartialEq)]
struct RedOrbExplosionLens {
    radius_start: f32,
    radius_end: f32,
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

impl Lens<RedOrbExplosion> for RedOrbExplosionLens {
    fn lerp(&mut self, target: &mut dyn Targetable<RedOrbExplosion>, ratio: f32) {
        target.radius = mix(self.radius_start, self.radius_end, ratio);
    }
}
