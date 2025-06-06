// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
// #[cfg(feature = "dev")]
mod dev_tools;
mod gas;
mod menus;
mod meteorites;
mod player;
mod screens;
mod space;
mod theme;

use avian2d::prelude::*;
use bevy::{
    asset::AssetMetaCheck, color::palettes::css::WHITE, core_pipeline::bloom::Bloom,
    diagnostic::FrameTimeDiagnosticsPlugin, prelude::*,
};
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_vector_shapes::Shape2dPlugin;

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
                        title: "Space Match 3".to_string(),
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
        ));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            // #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
            player::plugin,
            space::plugin,
            gas::plugin,
            meteorites::plugin,
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
