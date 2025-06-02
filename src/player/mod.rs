use bevy::{ecs::query::QueryFilter, prelude::*};

pub mod assets;
pub mod movement;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, spawn::plugin, assets::plugin));
}

#[derive(Component, QueryFilter)]
pub struct Player;
