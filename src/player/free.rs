use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use bevy::dev_tools::states::log_transitions;

#[cfg(feature = "dev")]
use crate::utils::toggle_vsync;

const TOGGLE_FREE_MODE_KEY: KeyCode = KeyCode::KeyF;
const TOGGLE_VSYNC_KEY: KeyCode = KeyCode::KeyV;

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
// #[states(scoped_entities)]
pub struct FreeMode(pub bool);

fn toggle_free_mode(
    current_state: Res<State<FreeMode>>,
    mut next_state: ResMut<NextState<FreeMode>>,
) {
    next_state.set(FreeMode(!current_state.get().0))
}

pub fn plugin(app: &mut App) {
    app.insert_state(FreeMode(false));

    #[cfg(feature = "dev")]
    app.add_systems(
        Update,
        toggle_free_mode.run_if(input_just_pressed(TOGGLE_FREE_MODE_KEY)),
    );

    #[cfg(feature = "dev")]
    app.add_systems(
        Update,
        toggle_vsync.run_if(input_just_pressed(TOGGLE_VSYNC_KEY)),
    );

    app.add_systems(Update, log_transitions::<FreeMode>);
}
