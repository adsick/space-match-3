use avian2d::prelude::LinearVelocity;
use bevy::{math::VectorSpace, prelude::*};

use crate::{
    player::{Player, movement::PlayerControls},
    screens::Screen,
};

// pub const INTRO_SHIP_DAMPING: f32 = 1.2;
// pub const NORMAL_SHIP_DAMPING

pub fn plugin(app: &mut App) {
    app.insert_state(IntroState(true))
        .insert_resource(IntroProgress { t: 0.0 })
        .add_systems(OnEnter(Screen::Gameplay), setup_intro)
        .add_systems(OnEnter(IntroState(false)), on_intro_finished)
        .add_systems(
            Update,
            camera_follow_player
                .run_if(in_state(Screen::Gameplay))
                .run_if(in_state(IntroState(true))),
        );
    // .add_systems(OnEnter(Screen::Gameplay), spawn_big_bang);
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct IntroState(pub bool);

#[derive(Resource)]
pub struct IntroProgress {
    t: f32,
}

fn setup_intro(
    mut player_controls: ResMut<PlayerControls>,
    mut intro_state: ResMut<NextState<IntroState>>,
) {
    println!("disabling controls");
    player_controls.enabled = false;
    intro_state.set(IntroState(true));
}

fn on_intro_finished(
    mut player_controls: ResMut<PlayerControls>,
    mut ship_velocity: Single<&mut LinearVelocity, With<Player>>,
) {
    println!("enabling controls");
    player_controls.enabled = true;

    ship_velocity.0 = Vec2::new(0.0, 400.);
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let f = (2.0 * t) - 2.0;
        0.5 * f * f * f + 1.0
    }
}

fn camera_follow_player(
    q_camera: Single<&mut Transform, With<Camera>>,
    q_player: Single<(&GlobalTransform, &LinearVelocity), With<Player>>,
    time: Res<Time>,

    mut intro_state: ResMut<NextState<IntroState>>,
    mut progress: ResMut<IntroProgress>,
) {
    const START_CAM_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, 10.);
    const END_CAM_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, 1100.);

    let t = ease_in_out_cubic(progress.t);

    let mut cam_transform = q_camera.into_inner();
    let (player_transform, vel) = q_player.into_inner();

    let interpolated_translation = START_CAM_TRANSLATION.lerp(END_CAM_TRANSLATION, t);

    *cam_transform =
        cam_transform.with_translation(player_transform.translation() + interpolated_translation);
    *cam_transform = cam_transform.looking_at(player_transform.translation(), Vec3::Y);

    progress.t += time.delta_secs() / 4.0;
    println!("intro progress: {}", progress.t);
    if progress.t >= 1.0 {
        progress.t = 0.0;
        intro_state.set(IntroState(false));
    }

    // cam_transform.translation = cam_transform.translation.move_towards(player_transform.translation().with_z(30.0 + 1.0 * vel_len), 45.0 * time.delta_secs());

    // cam_transform.translation = player_transform.translation().with_z(50.0 + 10.0 * vel_len);

    // cam_transform.translation.y += -vel_len / 4.0;

    // cam_transform.rotation = Quat::from_rotation_x(vel_len / 70.0);
}

// fn spawn_big_bang(mut commands: Commands) {
//
//     commands.spawn((
//
//     ))
// }
