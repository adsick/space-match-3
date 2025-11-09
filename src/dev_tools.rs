//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::PhysicsDebugPlugin;
use bevy::{
    color::palettes::css::WHITE,
    core_pipeline::bloom::Bloom,
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};

use bevy::{dev_tools::states::log_transitions, ui::UiDebugOptions};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use iyes_perf_ui::{
    PerfUiPlugin,
    entries::{
        PerfUiFixedTimeEntries, PerfUiFramerateEntries, PerfUiRenderEntries, PerfUiSystemEntries,
        PerfUiWindowEntries,
    },
    prelude::PerfUiRoot,
};

use crate::{screens::Screen, vfx};

const DEBUG_TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const PERFUI_TOGGLE_KEY: KeyCode = KeyCode::F3;
const BLOOM_TOGGLE_KEY: KeyCode = KeyCode::KeyB;

pub(super) fn plugin(app: &mut App) {
    // World inspector
    app.add_plugins((
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::default().run_if(resource_equals(WorldInspectorEnabled(true))),
    ));

    app.add_plugins((
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        bevy::render::diagnostic::RenderDiagnosticsPlugin,
        // bevy::diagnostic::FrameTimeDiagnosticsPlugin::default()
    ));

    app.add_plugins(PhysicsDebugPlugin::default());

    app.init_resource::<WorldInspectorEnabled>()
        .add_systems(Startup, spawn_perf_ui)
        .add_systems(
            Update,
            (|mut enabled: ResMut<WorldInspectorEnabled>| enabled.0 = !enabled.0)
                .run_if(input_just_pressed(KeyCode::KeyW).and(input_pressed(KeyCode::ControlLeft))),
        )
        .add_systems(Startup, spawn_grid);

    app.add_plugins(PerfUiPlugin);

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(DEBUG_TOGGLE_KEY)),
            toggle_perf_ui.run_if(input_just_pressed(PERFUI_TOGGLE_KEY)),
        ),
    );

    app.add_systems(
        Update,
        tooggle_bloom.run_if(input_just_pressed(BLOOM_TOGGLE_KEY)),
    );
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_perf_ui(mut query: Query<&mut Visibility, With<PerfUiRoot>>) {
    for mut visibility in &mut query {
        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}

fn spawn_perf_ui(mut commands: Commands) {
    commands.spawn(
        (
            // Contains everything related to FPS and frame time
            PerfUiFramerateEntries::default(),
            // CPU and GPU render times
            PerfUiRenderEntries::default(),
            // Contains everything related to the window and cursor
            PerfUiWindowEntries::default(),
            // Contains everything related to system diagnostics (CPU, RAM)
            PerfUiSystemEntries::default(),
            // Contains everything related to fixed timestep
            PerfUiFixedTimeEntries::default(),
            // ...
        ),
    );
}

#[derive(Default, PartialEq, Resource)]
pub struct WorldInspectorEnabled(bool);

fn spawn_grid(mut commands: Commands, mut gizmo_assets: ResMut<Assets<GizmoAsset>>) {
    let mut gizmo = GizmoAsset::default();

    gizmo.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::splat(100),
        Vec2::splat(20.),
        WHITE.with_alpha(0.02),
    );

    commands.spawn(Gizmo {
        handle: gizmo_assets.add(gizmo),
        line_config: GizmoLineConfig {
            width: 1.,
            ..default()
        },
        ..default()
    });
}

fn tooggle_bloom(
    mut commands: Commands,
    mut camera: Query<Entity, With<Camera>>,
    mut state: Local<bool>,
) {
    let camera = camera.single_mut().unwrap();
    let mut camera = commands.get_entity(camera).unwrap();

    if *state {
        info!("Enable Bloom");
        camera.insert(vfx::BASE_BLOOM);
    } else {
        info!("Disable Bloom");
        camera.remove::<Bloom>();
    }
    *state = !*state;
}
