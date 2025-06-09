use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, PlaybackState};

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<AudioAssets>();
    app.init_resource::<MusicHandle>();

    // app.add_systems(Update, play_loop.run_if(resource_added::<AudioAssets>));

    app.add_systems(OnEnter(Screen::Gameplay), resume_music);
    app.add_systems(OnExit(Screen::Gameplay), pause_music);
    app.add_systems(
        Update,
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[dependency]
    pub music: Handle<AudioSource>,
    #[dependency]
    pub explosion: Handle<AudioSource>,
    #[dependency]
    pub engine_fire: Handle<AudioSource>,
    #[dependency]
    pub lose: Handle<AudioSource>,
    #[dependency]
    pub sharp_1: Handle<AudioSource>,
    #[dependency]
    pub sharp_2: Handle<AudioSource>,
    #[dependency]
    pub tick_short: Handle<AudioSource>,
    #[dependency]
    pub tick_norm: Handle<AudioSource>,
    // #[dependency]
    // pub win_1: Handle<AudioSource>,
    #[dependency]
    pub win_2: Handle<AudioSource>,
    #[dependency]
    pub fm_snare: Handle<AudioSource>,
    #[dependency]
    pub pop_1: Handle<AudioSource>,
    #[dependency]
    pub pop_2: Handle<AudioSource>,
    #[dependency]
    pub pop_3: Handle<AudioSource>,
    #[dependency]
    pub button_click: Handle<AudioSource>,
    #[dependency]
    pub button_hover: Handle<AudioSource>,
    #[dependency]
    pub big_explosion: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/music.ogg"),
            explosion: assets.load("audio/sound_effects/explosion.ogg"),
            engine_fire: assets.load("audio/sound_effects/engine_fire.ogg"),
            lose: assets.load("audio/sound_effects/lose.ogg"),
            sharp_1: assets.load("audio/sound_effects/sharp_1.ogg"),
            sharp_2: assets.load("audio/sound_effects/sharp_2.ogg"),
            tick_short: assets.load("audio/sound_effects/tick_short.ogg"),
            tick_norm: assets.load("audio/sound_effects/tick_norm.ogg"),
            // win_1: assets.load("audio/sound_effects/win_1.ogg"),
            win_2: assets.load("audio/sound_effects/win_2.ogg"),
            fm_snare: assets.load("audio/sound_effects/fm_snare.ogg"),
            pop_1: assets.load("audio/sound_effects/pop_1.ogg"),
            pop_2: assets.load("audio/sound_effects/pop_2.ogg"),
            pop_3: assets.load("audio/sound_effects/pop_3.ogg"),
            button_click: assets.load("audio/sound_effects/button_click.ogg"),
            button_hover: assets.load("audio/sound_effects/button_hover.ogg"),
            big_explosion: assets.load("audio/sound_effects/big_explosion.ogg"),
        }
    }
}

#[derive(Resource, Default)]
pub struct MusicHandle(Handle<AudioInstance>);

fn pause_music(mut audio_instances: ResMut<Assets<AudioInstance>>, handle: Res<MusicHandle>) {
    let Some(music) = audio_instances.get_mut(&handle.0) else {
        return;
    };

    music.pause(AudioTween::default());
}

fn resume_music(mut audio_instances: ResMut<Assets<AudioInstance>>, handle: Res<MusicHandle>) {
    let Some(music) = audio_instances.get_mut(&handle.0) else {
        return;
    };

    music.resume(AudioTween::default());
}

fn play_loop(mut commands: Commands, assets: Res<AudioAssets>, audio: Res<Audio>) {
    debug!("playing loop...");
    let handle = audio
        .play(assets.music.clone())
        .paused()
        .with_volume(0.3)
        .looped()
        .handle();
    commands.insert_resource(MusicHandle(handle));
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(audio: Res<Audio>) {
    // audio.set_volume(GlobalVolume::get());
}
