use bevy::prelude::*;
use bevy_tweening::{AnimCompletedEvent, CycleCompletedEvent, TweenAnim, TweenResolver};

// copied from bevy_tweening itself. this is needed in order to move Tweening to other schedule

/// Impostor
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TweenResolver>()
            .add_message::<CycleCompletedEvent>()
            .add_message::<AnimCompletedEvent>()
            .add_systems(
                FixedUpdate,
                animator_system.in_set(AnimationSystem::AnimationUpdate),
            );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
#[non_exhaustive]
pub enum AnimationSystem {
    /// Steps all animations. This executes during the [`Update`] schedule.
    AnimationUpdate,
}

/// Core animation system ticking all queued animations.
///
/// This calls [`TweenAnim::step_all()`] using a value of the animation timestep
/// `delta_time` equal to [`Time::delta()`].
pub(crate) fn animator_system(world: &mut World) {
    let delta_time = world.resource::<Time>().delta();
    TweenAnim::step_all(world, delta_time);
}
