use avian2d::prelude::*;
use rand::Rng;

use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;

pub const BASE_BLOOM: Bloom = Bloom::NATURAL;

pub fn screen_shake_plugin(app: &mut App) {
    app.init_resource::<ScreenShake>();
    app.add_systems(Update, screen_shake);
}

#[derive(Default, Resource, Clone)]
pub struct ScreenShake {
    max_angle: f32,
    max_offset: f32,
    trauma: f32,
    last_position: Vec2,
    current_position: Vec2,
    pub until: f32, // physics time in seconds
}

const CAMERA_DECAY_RATE: f32 = 0.9; // Adjust this for smoother or snappier decay
const TRAUMA_DECAY_SPEED: f32 = 0.5; // How fast trauma decays

// stolen from https://bevy.org/examples/camera/2d-screen-shake/
pub fn screen_shake(
    time: Res<Time<Physics>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if time.elapsed_secs() < screen_shake.until {
        // * maybe tweak these
        screen_shake.max_angle = 0.5;
        screen_shake.max_offset = 500.0;
        screen_shake.trauma = (screen_shake.trauma + 1.0 * time.delta_secs()).clamp(0.0, 1.0);
        screen_shake.last_position = Vec2::new(0.0, 0.0);
    }

    let mut rng = rand::thread_rng();
    let shake = screen_shake.trauma * screen_shake.trauma;
    let angle = (screen_shake.max_angle * shake).to_radians() * rng.gen_range(-1.0..1.0);
    let offset_x = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);
    let offset_y = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);

    if shake > 0.0 {
        for mut transform in query.iter_mut() {
            // Position
            let target = screen_shake.current_position
                + Vec2 {
                    x: offset_x,
                    y: offset_y,
                };
            screen_shake.current_position.smooth_nudge(
                &target,
                CAMERA_DECAY_RATE,
                time.delta_secs(),
            );

            transform.translation += target.extend(0.0) * time.delta_secs() * 5.0; // * maybe change 5.0 here and below to a different value for strength

            // Rotation
            let rotation = Quat::from_rotation_z(angle);
            transform.rotation = transform
                .rotation
                .interpolate_stable(&(transform.rotation.mul_quat(rotation)), CAMERA_DECAY_RATE);
        }
    } else if let Ok(mut transform) = query.single_mut() {
        let target = screen_shake.last_position;
        screen_shake
            .current_position
            .smooth_nudge(&target, 1.0, time.delta_secs());

        transform.translation += target.extend(0.0) * time.delta_secs() * 5.0;

        transform.rotation = transform.rotation.interpolate_stable(&Quat::IDENTITY, 0.1);
    }
    // Decay the trauma over time
    screen_shake.trauma -= TRAUMA_DECAY_SPEED * time.delta_secs();
    screen_shake.trauma = screen_shake.trauma.clamp(0.0, 1.0);
}
