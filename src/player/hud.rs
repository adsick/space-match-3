use bevy::{prelude::*, ui::Val::*};

use crate::{player::Player, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup_hud);
    app.add_systems(Update, update_hud);
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Hud"),
        StateScoped(Screen::Gameplay),
        Node {
            width: Percent(100.0),
            height: Percent(100.0),

            ..default()
        },
        children![(
            widget::label("Score: "),
            Node {
                left: Val::Px(10.0),
                top: Val::Px(10.0),

                ..default()
            },
            HudScores,
        )],
    ));
}

fn update_hud(player: Single<&Player>, mut text: Single<&mut Text, With<HudScores>>) {
    text.0 = format!(
        "Score: {}\nStyle Points: {}",
        player.score as u32, player.style_points
    );
}

#[derive(Component)]
pub struct HudScores;
