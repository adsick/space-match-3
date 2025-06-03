use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub ship: Handle<Image>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ship: assets.load("images/ship.png"),
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<PlayerAssets>()
        .load_resource::<PlayerAssets>();
}
