use bevy::{
    color::palettes::css::{WHEAT, WHITE},
    prelude::*,
    render::mesh::CircleMeshBuilder,
};

use crate::asset_tracking::LoadResource;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct OrbAssets {
    #[dependency]
    pub orb_meshes: Vec<Handle<Mesh>>,
    #[dependency]
    pub orb_materials: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for OrbAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let mut orb_meshes = vec![];
        let mut orb_materials = vec![];

        for r in 6..10 {
            orb_meshes.push(assets.add(CircleMeshBuilder::new(1.0, r).build()));

            orb_materials.push(assets.add(StandardMaterial {
                base_color: WHEAT.with_alpha(14.0 / (1.0 + r as f32 * 20.0)).into(),
                alpha_mode: AlphaMode::Blend,
                emissive: (WHITE * 1.0).into(),
                ..Default::default()
            }));
        }

        Self {
            orb_meshes,
            orb_materials,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<OrbAssets>()
        .load_resource::<OrbAssets>();
}
