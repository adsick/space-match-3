use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_real_timer};
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioTween, prelude::Volume};

use crate::{
    audio::AudioAssets,
    player::Player,
    red_gas::{RedOrbExplosion, RedOrbExplosionEvent},
};

const UPDATE_RATE: Duration = Duration::from_millis(500);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            play_explosion_sound.run_if(resource_added::<AudioAssets>),
            update_volume
                .run_if(resource_exists::<RedOrbExplosionSound>.and(on_real_timer(UPDATE_RATE))),
        ),
    );
}

#[derive(Resource)]
pub struct RedOrbExplosionSound(Handle<AudioInstance>);

fn play_explosion_sound(audio: Res<Audio>, audio_assets: Res<AudioAssets>, mut cmds: Commands) {
    cmds.insert_resource(RedOrbExplosionSound(
        audio
            .play(audio_assets.big_explosion.clone())
            .with_volume(0.0)
            .looped()
            .handle(),
    ));
}

fn update_volume(
    sound: Res<RedOrbExplosionSound>,
    q_player: Query<&GlobalTransform, With<Player>>,
    mut sounds: ResMut<Assets<AudioInstance>>,
    q_explosions: Query<(&GlobalTransform, &RedOrbExplosion)>,
) {
    let Some(instance) = sounds.get_mut(sound.0.id()) else {
        return;
    };
    let Ok(player_transform) = q_player.single() else {
        instance.set_volume(0.0, AudioTween::default());
        return;
    };
    let player_pos = player_transform.translation().truncate();
    let highest_volume = q_explosions
        .iter()
        .map(|(tr, orb)| {
            1.0 / ((player_pos.distance(tr.translation().truncate()) - orb.radius).max(0.0) / 100.0
                + 1.0)
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    instance.set_volume(
        Volume::Amplitude(highest_volume as f64 * 0.6),
        AudioTween::linear(UPDATE_RATE),
    );
}
