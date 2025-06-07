//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
    ui::UiDebugOptions,
};
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
    .init_resource::<WorldInspectorEnabled>()
    .add_systems(Startup, spawn_perf_ui)
    .add_systems(
        Update,
        (|mut enabled: ResMut<WorldInspectorEnabled>| enabled.0 = !enabled.0)
            .run_if(input_just_pressed(KeyCode::KeyW).and(input_pressed(KeyCode::ControlLeft))),
    );

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
}

const DEBUG_TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const PERFUI_TOGGLE_KEY: KeyCode = KeyCode::F3;

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
