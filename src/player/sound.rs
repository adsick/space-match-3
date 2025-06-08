use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::{
    Audio, AudioControl, AudioEasing, AudioInstance, AudioTween, SpatialAudioEmitter,
    prelude::Volume,
};

use crate::{audio::AudioAssets, player::Player};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_sound)
        .add_observer(stop_sound)
        .add_systems(Update, update_sound);
}

#[derive(Component)]
pub struct EngineSound(Handle<AudioInstance>);

const VOLUME: f32 = 0.3;

fn setup_sound(
    trigger: Trigger<OnAdd, Player>,
    mut cmds: Commands,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    let sound = audio
        .play(audio_assets.engine_fire.clone())
        .with_volume(Volume::Amplitude(VOLUME as f64))
        .looped()
        .handle();
    cmds.entity(trigger.target()).insert(EngineSound(sound));
}

fn update_sound(
    q_sound: Single<&EngineSound>,
    q_camera: Single<&GlobalTransform, With<Camera>>,
    mut sounds: ResMut<Assets<AudioInstance>>,
    time: Res<Time>,
) {
    let sound = q_sound.into_inner();
    let cam_tr = q_camera.into_inner();
    let Some(instance) = sounds.get_mut(sound.0.id()) else {
        return;
    };
    instance.set_volume(
        Volume::Amplitude((VOLUME / (cam_tr.translation().z / 500.0 + 1.0)) as f64),
        AudioTween::linear(Duration::from_secs_f32(time.delta_secs())),
    );
}

fn stop_sound(
    trigger: Trigger<OnRemove, EngineSound>,
    q_sound: Query<&EngineSound>,
    mut sounds: ResMut<Assets<AudioInstance>>,
) {
    let Ok(sound) = q_sound.get(trigger.target()) else {
        return;
    };
    let Some(instance) = sounds.get_mut(sound.0.id()) else {
        return;
    };
    instance.stop(AudioTween::linear(Duration::from_secs(1)));
}
