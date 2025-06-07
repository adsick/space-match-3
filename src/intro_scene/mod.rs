use avian2d::prelude::LinearVelocity;
use bevy::{
    app::{App, Update},
    prelude::{Commands, IntoScheduleConfigs, OnEnter, Query},
};

use crate::{
    player::{Player, movement::PlayerControls},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), start_intro_scene);
}

fn start_intro_scene(
    mut commands: Commands,
    player: Query<(&Player, &LinearVelocity, &PlayerControls)>,
) {
}
