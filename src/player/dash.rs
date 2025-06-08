use avian2d::prelude::{ExternalForce, Rotation};
use bevy::{math::VectorSpace, prelude::*, transform::commands};

use crate::{screens::Screen, space::intro::IntroState};

use super::Player;

const DASH_STRENGTH: f32 = 50000.0;
const DASH_DURATION_MILLIS: u128 = 200;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_added).add_systems(
        Update,
        side_dash
            .run_if(in_state(Screen::Gameplay))
            .run_if(not(in_state(IntroState(true)))),
    );
}

#[derive(Component)]
struct DashData {
    curr_dash: Option<Dash>,
}

struct Dash {
    force: Vec2,
    start_t_millis: u128,
}

fn on_player_added(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(DashData { curr_dash: None });
}

fn side_dash(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Single<(&mut ExternalForce, &Rotation, &mut Player, &mut DashData)>,
    time: Res<Time>,
) {
    let (mut force, rotation, mut player, mut dash_data) = player_query.into_inner();

    let elapsed_millis = time.elapsed().as_millis();
    if let Some(dash) = &dash_data.curr_dash {
        if elapsed_millis > dash.start_t_millis + DASH_DURATION_MILLIS {
            force.apply_force(-dash.force);
            dash_data.curr_dash = None;
        }
    }

    if !player.dash_timer.tick(time.delta()).finished() {
        return;
    }

    let mut f: Option<Vec2> = None;
    if keyboard_input.pressed(KeyCode::KeyQ) {
        f = Some(rotation * Vec2::X * -DASH_STRENGTH);
    } else if keyboard_input.pressed(KeyCode::KeyE) {
        f = Some(rotation * Vec2::X * DASH_STRENGTH);
    }

    if let Some(f) = f {
        dash_data.curr_dash = Some(Dash {
            force: f,
            start_t_millis: elapsed_millis,
        });
        force.apply_force(f);
        player.dash_timer.reset();
    }
}
