use bevy::{
    app::App,
    asset::{Asset, AssetServer, Assets, Handle},
    color::{
        Alpha,
        palettes::css::{RED, WHEAT, WHITE},
    },
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Commands, Component, FromWorld, Mesh, Mesh3d, MeshBuilder, OnAdd, Query, Res,
        ResMut, Resource, Transform, Trigger,
    },
    reflect::Reflect,
    render::mesh::{CircleMeshBuilder, SphereKind, SphereMeshBuilder},
};

pub fn plugin(app: &mut App) {
    app.add_observer(on_add_explosive_gas_orb)
        .init_resource::<ExplosiveOrbAssets>();
}

#[derive(Component)]
pub struct ExplosiveGasOrb {
    pub radius: f32,
    pub pos: Vec3,
}

#[derive(Resource, Asset, Clone, Reflect)]
struct ExplosiveOrbAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    explosion_materials: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for ExplosiveOrbAssets {
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

        let explosion_materials = vec![
            assets.add(StandardMaterial {
                base_color: WHEAT.with_alpha(0.5).into(),
                alpha_mode: AlphaMode::Blend,
                emissive: (WHITE * 1.0).into(),
                ..Default::default()
            }),
            assets.add(StandardMaterial {
                base_color: WHEAT.with_alpha(0.5).into(),
                alpha_mode: AlphaMode::Blend,
                emissive: (WHITE * 1.0).into(),
                ..Default::default()
            }),
        ];

        ExplosiveOrbAssets {
            mesh,
            material,
            explosion_materials,
        }
    }
}

fn on_add_explosive_gas_orb(
    trigger: Trigger<OnAdd, ExplosiveGasOrb>,
    mut commands: Commands,
    orbs: Query<&ExplosiveGasOrb>,
    orb_assets: Res<ExplosiveOrbAssets>,
) {
    let entity = trigger.target();
    let Ok(orb) = orbs.get(entity) else {
        return;
    };

    let mut transform = Transform::from_translation(orb.pos).with_scale(Vec3::splat(orb.radius));

    transform.rotate_local_z(orb.pos.x);
    transform.rotate_local_x(orb.pos.y);

    commands.entity(entity).insert((
        transform,
        Mesh3d(orb_assets.mesh.clone()),
        MeshMaterial3d(orb_assets.material.clone()),
    ));
}
