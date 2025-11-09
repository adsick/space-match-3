use std::f32::consts::PI;

use avian2d::prelude::{ExternalForce, LinearVelocity, Rotation};
use bevy::{math::VectorSpace, prelude::*, transform::commands};

use crate::{screens::Screen, space::intro::IntroState};

use super::Player;

const DASH_STRENGTH: f32 = 20000.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_added)
        .add_systems(Update, side_dash.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
struct DashData {
    dash_timer: DashTimer,
}

#[derive(Component, Deref, DerefMut, Reflect, Default)]
pub struct DashTimer(pub Timer);

fn on_player_added(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(DashData {
        dash_timer: DashTimer(Timer::from_seconds(1., TimerMode::Once)),
    });
}

fn side_dash(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Single<(&mut ExternalForce, &Rotation, &mut DashData)>,
    time: Res<Time>,
) {
    let (mut force, rotation, mut dash_data) = player_query.into_inner();

    if !dash_data.dash_timer.tick(time.delta()).finished() {
        return;
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        force.apply_force((*rotation) * Vec2::X * -DASH_STRENGTH);

        dash_data.dash_timer.reset();
    } else if keyboard_input.pressed(KeyCode::KeyE) {
        force.apply_force((*rotation) * Vec2::X * DASH_STRENGTH);

        dash_data.dash_timer.reset();
    }
}
