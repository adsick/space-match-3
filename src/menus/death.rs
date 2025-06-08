use bevy::{color::palettes::css::BLACK, input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Death), spawn_death_menu);
}

fn spawn_death_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("DEAD"),
        GlobalZIndex(2),
        StateScoped(Menu::Death),
        children![
            (
                Text("DEAD".into()),
                TextFont::from_font_size(80.0),
                TextColor(BLACK.into()),
            ),
            // TODO: display the score
            widget::button("Restart", restart),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn restart(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    // TODO:
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
