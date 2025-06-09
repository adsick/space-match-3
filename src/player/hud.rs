use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    PausableSystems,
    asset_tracking::LoadResource,
    player::{Player, Score, movement::AuraEarned},
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HudAssets>()
        .load_resource::<HudAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), setup_hud);
    app.add_systems(
        Update,
        update_hud
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn setup_hud(mut commands: Commands, assets: Res<HudAssets>) {
    commands.spawn((
        widget::ui_root_uncentered("Scores"),
        StateScoped(Screen::Gameplay),
        children![(
            widget::label("Score:\nAura:"),
            Node {
                align_self: AlignSelf::Start,
                left: Val::Px(10.0),
                top: Val::Px(10.0),

                ..default()
            },
            HudScores,
        )],
    ));

    commands.spawn((
        widget::ui_root_uncentered("Abilities"),
        StateScoped(Screen::Gameplay),
        children![(
            widget::emoji_label("⏳", &assets),
            Node {
                align_self: AlignSelf::End,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),

                ..default()
            },
            HudAbilities,
        )],
    ));
}

// TODO: DASH ABILITY
fn update_hud(
    player: Single<&Player>,
    mut score_text: Single<&mut Text, (With<HudScores>, Without<HudAbilities>)>,
    mut abilities_text: Single<&mut Text, (With<HudAbilities>, Without<HudScores>)>,
    score: Res<Score>,
    time: Res<Time>,
    mut aura_event: EventReader<AuraEarned>,
    mut recent_earnings: Local<VecDeque<(f32, u32)>>,
) {
    let mut earnings = String::new();
    let ct = time.elapsed().as_millis() as u32;


    let mut earned = 0.0;
    for ev in aura_event.read() {
        earned += ev.0;
    }

    if earned > 0.0 {
        if recent_earnings.len() == 0 {
            recent_earnings.push_front((earned, ct));
        } else if ct - recent_earnings[0].1 < 40 {
            recent_earnings[0].0 += earned;
            recent_earnings[0].1 = ct;
        } else {
            recent_earnings.push_front((earned, ct));
            recent_earnings.truncate(3);
        }
    }

    for (ev, t) in &*recent_earnings {
        if ct - t > 3000 {
            break;
        }
        earnings.push_str(&format!("     +{ev:.0}\n"));
    }

    score_text.0 = format!(
        "Score: {:.1}\nAura: {}\n{earnings}",
        score.0, player.aura_points as i32,
    );
    let mut abilities_string = "Bullet time:".to_string();
    let elapsed = time.elapsed_secs();
    let mut has_abilities = false;
    if player.bullet_time_cooldown_until < elapsed && player.bullet_time_until < elapsed {
        abilities_string.push('⏳');
        has_abilities = true;
    }
    if !has_abilities {
        abilities_string.push('❌');
    }
    // if player.dash_timer.0.finished() {
    //     abilities_string.push('💖');
    // } else {
    //     abilities_string.push('💔');
    // }

    abilities_text.0 = abilities_string;
}

#[derive(Component)]
pub struct HudScores;

#[derive(Component)]
pub struct HudAbilities;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct HudAssets {
    #[dependency]
    pub font: Handle<Font>,
}

impl FromWorld for HudAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            font: assets.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
        }
    }
}
