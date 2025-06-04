use bevy::{
    color::palettes::css::{WHEAT, WHITE},
    prelude::*,
    render::mesh::CircleMeshBuilder,
    sprite::Material2d,
};

use crate::asset_tracking::LoadResource;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SpaceAssets {
    #[dependency]
    pub orb_meshes: Vec<Handle<Mesh>>,
    #[dependency]
    pub orb_materials: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for SpaceAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let mut orb_meshes = vec![];
        let mut orb_materials = vec![];

        for r in 6..10 {
            orb_meshes.push(assets.add(CircleMeshBuilder::new(1.0, r).build()));

            orb_materials.push(assets.add(StandardMaterial {
                base_color: WHEAT.with_alpha(4.0 / (1.0 + r as f32 * 20.0)).into(),
                // alpha_mode: AlphaMode::AlphaToCoverage,
                emissive: (WHITE * 0.5).into(),
                ..Default::default()
            }));
        }

        Self {
            orb_meshes,
            orb_materials,
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<SpaceAssets>()
        .load_resource::<SpaceAssets>();
}
