//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
    ui::UiDebugOptions,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // World inspector
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(resource_equals(WorldInspectorEnabled(true))),
    )
    .init_resource::<WorldInspectorEnabled>()
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
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

#[derive(Default, PartialEq, Resource)]
pub struct WorldInspectorEnabled(bool);
