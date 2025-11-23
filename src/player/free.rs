use bevy::{input::common_conditions::input_just_pressed, prelude::*};

const TOGGLE_FREE_MODE_KEY: KeyCode = KeyCode::KeyG;

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
    app.add_systems(
        Update,
        toggle_free_mode.run_if(input_just_pressed(TOGGLE_FREE_MODE_KEY)),
    );
}
