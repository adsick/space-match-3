use bevy::{
    color::palettes::{
        css::{GOLD, RED, WHEAT, WHITE},
        tailwind::GRAY_700,
    },
    prelude::*,
    render::mesh::CircleMeshBuilder,
};

use crate::asset_tracking::LoadResource;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct OrbAssets {
    #[dependency]
    pub orb_mesh: Handle<Mesh>,
    #[dependency]
    pub orb_materials: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for OrbAssets {
    fn from_world(world: &mut World) -> Self {
        println!("initing orb assets");
        let assets = world.resource::<AssetServer>();

        let mut orb_materials = vec![];

        let orb_mesh = assets.add(CircleMeshBuilder::new(1.0, 12).build());

        orb_materials.push(assets.add(StandardMaterial {
            base_color: WHEAT.with_alpha(0.5).into(),
            alpha_mode: AlphaMode::Blend,
            emissive: (WHITE * 1.0).into(),
            ..Default::default()
        }));

        orb_materials.push(assets.add(StandardMaterial {
            base_color: RED.into(),
            alpha_mode: AlphaMode::Opaque,
            emissive: (GOLD * 2.0).into(),
            ..Default::default()
        }));

        orb_materials.push(assets.add(StandardMaterial {
            base_color: RED.with_alpha(0.7).into(),
            alpha_mode: AlphaMode::Blend,
            emissive: (RED * 0.5).mix(&GOLD, 0.3).into(),
            ..Default::default()
        }));

        orb_materials.push(assets.add(StandardMaterial {
            base_color: GRAY_700.with_alpha(0.2).into(),
            alpha_mode: AlphaMode::Multiply,
            // emissive: (WHITE * 0.8).into(),
            ..Default::default()
        }));

        Self {
            orb_mesh,
            orb_materials,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    println!("registering orb assets");
    app.register_type::<OrbAssets>()
        .load_resource::<OrbAssets>();
}
