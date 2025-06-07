// Support configuring Bevy lints withinv code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

#![allow(unused_imports)]
mod asset_tracking;
mod audio;
// #[cfg(feature = "dev")]
mod asteroids;
mod dev_tools;
mod intro_scene;
mod menus;
mod player;
mod red_gas;
mod screens;
mod space;
mod theme;
mod utils;

use avian2d::prelude::*;
use bevy::{
    asset::AssetMetaCheck, color::palettes::css::WHITE, core_pipeline::bloom::Bloom, diagnostic::FrameTimeDiagnosticsPlugin, math::VectorSpace, prelude::*
};
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_tweening::TweeningPlugin;
use bevy_vector_shapes::Shape2dPlugin;
use rand::Rng;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Space rush - complete burnout".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            PhysicsPlugins::default().with_length_unit(1.0),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            FramepacePlugin,
            Shape2dPlugin::default(), // bevy_vector_shapes
            TweeningPlugin,
        ));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            // #[cfg(feature = "dev")]
            dev_tools::plugin,
            asteroids::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
            player::plugin,
            space::plugin,
            red_gas::plugin,
            utils::plugin,
            intro_scene::plugin,
            FrameTimeDiagnosticsPlugin::default(),
        ));

        app.insert_resource(ClearColor(Color::srgb(0.12, 0.1, 0.14)));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // screen shake shit
        app.init_resource::<CameraShake>();
        app.add_systems(Update, screen_shake);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

#[derive(Default, Resource, Clone)]
struct CameraShake {
    max_angle: f32,
    max_offset: f32,
    trauma: f32,
    last_position: Vec2,
    current_position: Vec2,
    until: f32, // physics time in seconds
}

const CAMERA_DECAY_RATE: f32 = 0.9; // Adjust this for smoother or snappier decay
const TRAUMA_DECAY_SPEED: f32 = 0.5; // How fast trauma decays

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera {
            hdr: true,
            ..default()
        },
        Camera3d::default(),
        Bloom::NATURAL,
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 500.0,
            shadows_enabled: false,
            ..Default::default()
        },
        Transform::default().looking_at(Vec3::new(0.0, 10.0, -2.0), Dir3::Y),
    ));
}

#[derive(PhysicsLayer, Default)]
pub enum PhysicsLayers {
    #[default]
    Default,
    RedOrbs,
    RedOrbExplosions,
}

// TODO: move this somewhere else?
fn screen_shake(
    time: Res<Time<Physics>>,
    mut screen_shake: ResMut<CameraShake>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if time.elapsed_secs() < screen_shake.until {
        // * maybe tweak these
        screen_shake.max_angle = 0.5;
        screen_shake.max_offset = 500.0;
        screen_shake.trauma = (screen_shake.trauma + 1.0 * time.delta_secs()).clamp(0.0, 1.0);
        screen_shake.last_position = Vec2::new(0.0, 0.0);
    }

    let mut rng = rand::thread_rng();
    let shake = screen_shake.trauma * screen_shake.trauma;
    let angle = (screen_shake.max_angle * shake).to_radians() * rng.gen_range(-1.0..1.0);
    let offset_x = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);
    let offset_y = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);

    if shake > 0.0 {
        for mut transform in query.iter_mut() {
            // Position
            let target = screen_shake.current_position
                + Vec2 {
                    x: offset_x,
                    y: offset_y,
                };
            screen_shake.current_position.smooth_nudge(
                &target,
                CAMERA_DECAY_RATE,
                time.delta_secs(),
            );

            transform.translation += target.extend(0.0) * time.delta_secs() * 5.0; // * maybe change 5.0 here and below to a different value for strength

            // Rotation
            let rotation = Quat::from_rotation_z(angle);
            transform.rotation = transform
                .rotation
                .interpolate_stable(&(transform.rotation.mul_quat(rotation)), CAMERA_DECAY_RATE);
        }
    } else if let Ok(mut transform) = query.single_mut() {
        let target = screen_shake.last_position;
        screen_shake
            .current_position
            .smooth_nudge(&target, 1.0, time.delta_secs());

        transform.translation += target.extend(0.0) * time.delta_secs() * 5.0;

        transform.rotation = transform.rotation.interpolate_stable(&Quat::IDENTITY, 0.1);
    }
    // Decay the trauma over time
    screen_shake.trauma -= TRAUMA_DECAY_SPEED * time.delta_secs();
    screen_shake.trauma = screen_shake.trauma.clamp(0.0, 1.0);
}
