use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub ship: Handle<Mesh>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ship: assets.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 0,
                }
                .from_asset("3D/Ship1.glb"),
            ),
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<PlayerAssets>()
        .load_resource::<PlayerAssets>();
}
