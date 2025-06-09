use bevy::{
    color::palettes::css::RED,
    prelude::*,
    render::mesh::{CircleMeshBuilder, SphereKind, SphereMeshBuilder},
};

#[derive(Resource, Asset, Clone, Reflect)]
pub struct RedOrbAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub explosion_mesh: Handle<Mesh>,
    pub explosion_material: Handle<StandardMaterial>,
}

impl FromWorld for RedOrbAssets {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let assets = world.resource::<AssetServer>();

        let mesh =
            assets.add(SphereMeshBuilder::new(1.0, SphereKind::Ico { subdivisions: 3 }).build());

        let material = assets.add(StandardMaterial {
            base_color: RED.with_alpha(0.7).into(),
            alpha_mode: AlphaMode::Blend,
            emissive: (RED * 5.0).into(),
            ..Default::default()
        });

        // let explosion_materials = vec![
        //     assets.add(StandardMaterial {
        //         base_color: RED.with_alpha(0.5).into(),
        //         alpha_mode: AlphaMode::Blend,
        //         emissive: (RED * 1.0).into(),
        //         fog_enabled: false,
        //         ..Default::default()
        //     }),
        //     assets.add(StandardMaterial {
        //         base_color: RED.with_alpha(0.5).into(),
        //         alpha_mode: AlphaMode::Blend,
        //         emissive: (RED * 3.0).into(),
        //         fog_enabled: false,
        //         ..Default::default()
        //     }),
        // ];

        let explosion_material = assets.add(StandardMaterial {
            base_color: RED.with_alpha(0.7).into(),
            alpha_mode: AlphaMode::Blend,
            emissive: (RED * 10.0).into(),
            ..Default::default()
        });
        let explosion_mesh = assets.add(CircleMeshBuilder::new(1.0, 64).build());

        RedOrbAssets {
            mesh,
            material,
            explosion_material,
            explosion_mesh,
        }
    }
}
