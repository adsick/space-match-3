//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::PhysicsDebugPlugin;
use bevy::{
    color::palettes::css::WHITE,
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};

#[cfg(feature = "dev")]
use bevy::{dev_tools::states::log_transitions, ui::UiDebugOptions};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_perf_ui::{
    PerfUiPlugin,
    prelude::{PerfUiDefaultEntries, PerfUiRoot},
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // World inspector
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(resource_equals(WorldInspectorEnabled(true))),
    )
    .add_plugins(PerfUiPlugin)
    .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
    .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
    .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
    .add_plugins(PhysicsDebugPlugin::default())
    .init_resource::<WorldInspectorEnabled>()
    .add_systems(Startup, spawn_perf_ui)
    .add_systems(
        Update,
        (|mut enabled: ResMut<WorldInspectorEnabled>| enabled.0 = !enabled.0)
            .run_if(input_just_pressed(KeyCode::KeyW).and(input_pressed(KeyCode::ControlLeft))),
    )
    .add_systems(Startup, spawn_grid);

    // Log `Screen` state transitions.
    #[cfg(feature = "dev")]
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (
            #[cfg(feature = "dev")]
            toggle_debug_ui.run_if(input_just_pressed(DEBUG_TOGGLE_KEY)),
            toggle_perf_ui.run_if(input_just_pressed(PERFUI_TOGGLE_KEY)),
        ),
    );
}

const DEBUG_TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const PERFUI_TOGGLE_KEY: KeyCode = KeyCode::F3;

#[cfg(feature = "dev")]
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
    commands.spawn(PerfUiDefaultEntries::default());
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
