use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AudioAssets>();
    app.load_resource::<AudioAssets>();

    app.add_systems(Update, play_music.run_if(resource_added::<AudioAssets>));
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
    #[dependency]
    pub win_1: Handle<AudioSource>,
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
            win_1: assets.load("audio/sound_effects/win_1.ogg"),
            win_2: assets.load("audio/sound_effects/win_2.ogg"),
            fm_snare: assets.load("audio/sound_effects/fm_snare.ogg"),
            pop_1: assets.load("audio/sound_effects/pop_1.ogg"),
            pop_2: assets.load("audio/sound_effects/pop_2.ogg"),
            pop_3: assets.load("audio/sound_effects/pop_3.ogg"),
            button_click: assets.load("audio/sound_effects/button_click.ogg"),
            button_hover: assets.load("audio/sound_effects/button_hover.ogg"),
        }
    }
}

fn play_music(assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.play(assets.music.clone()).with_volume(0.3).looped();
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(audio: Res<Audio>) {
    // audio.set_volume(GlobalVolume::get());
}
