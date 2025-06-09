use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

use crate::{audio::AudioAssets, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Dead), spawn_death_menu);
}

fn spawn_death_menu(mut commands: Commands, audio: Res<Audio>, audio_assets: Res<AudioAssets>) {
    audio.play(audio_assets.lose.clone()).with_volume(0.7);
    commands.spawn((
        widget::ui_root("DEAD"),
        // GlobalZIndex(1),
        StateScoped(Screen::Dead),
        children![
            widget::header("DEAD"),
            // TODO: display the score
            widget::button("Restart", restart),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn restart(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
