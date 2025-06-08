use std::collections::VecDeque;

use bevy::prelude::*;

use crate::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, render_speed_tracers);
}

const TRACER_COUNT: usize = 200;
const TRACER_LENGTH: usize = 5;
const WRAPPING_HALF_EXTENT: f32 = 1000.0; // Increase this if lines are visibly teleporting
const RENDER_STRIDE: usize = 1; // Reduces rendering cost at the risk of making lines less smooth
const MIN_Z: f32 = 0.0;
const MAX_Z: f32 = 700.0;
const Z_EXTENT: f32 = MAX_Z - MIN_Z;
const MIDDLE_Z: f32 = (MIN_Z + MAX_Z) / 2.0;

fn render_speed_tracers(
    q_player: Single<&Transform, With<Player>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
    mut history: Local<VecDeque<GlobalTransform>>,
    mut points: Local<Option<Vec<(Vec3, Srgba)>>>,
) {
    let transform = q_player.into_inner();
    let (cam, cam_tr) = q_camera.into_inner();

    let points = points.get_or_insert_with(|| {
        (0..TRACER_COUNT)
            .map(|_| {
                (
                    Vec3::new(
                        (rand::random::<f32>() * 2.0 - 1.0) * WRAPPING_HALF_EXTENT,
                        (rand::random::<f32>() * 2.0 - 1.0) * WRAPPING_HALF_EXTENT,
                        rand::random::<f32>() * Z_EXTENT,
                    ),
                    // Blend between red-ish and blue-ish
                    (Srgba::rgb(1.0, 0.7, 0.6) * 1.5)
                        .mix(&(Srgba::rgb(0.9, 0.9, 1.0) * 2.0), rand::random()),
                )
            })
            .collect()
    });

    // Find the new world position of an object assuming position on screen hasn't changed
    let reproject = |old_tr: &GlobalTransform, pos: Vec3| {
        let screen_pos = cam.world_to_viewport(old_tr, pos).ok()?;
        let ray = cam.viewport_to_world(&cam_tr, screen_pos).ok()?;
        let dist = ray.intersect_plane(pos, InfinitePlane3d::new(Vec3::Z))?;
        Some(ray.get_point(dist))
    };

    let center = transform.translation.truncate();
    history.push_front(*cam_tr);
    history.truncate(TRACER_LENGTH);
    for (pos, color) in points {
        // wrap points back when they exceed limits
        *pos = ((pos.truncate() - center).map(|x| {
            (x - WRAPPING_HALF_EXTENT / 2.0).rem_euclid(WRAPPING_HALF_EXTENT)
                - WRAPPING_HALF_EXTENT / 2.0
        }) + center)
            .extend(pos.z);

        // wrap points back on the z axis
        if cam_tr.translation().z - pos.z > MAX_Z {
            pos.z += Z_EXTENT;
        }
        if cam_tr.translation().z - pos.z < MIN_Z {
            pos.z -= Z_EXTENT;
        }

        // fade out the further it is from the middle of the range, the more opaque
        let opacity =
            ((cam_tr.translation().z - pos.z - MIDDLE_Z).abs() / (Z_EXTENT / 2.0)).powf(1.0 / 1.0);

        // gizmos.sphere(*pos, 2.0, *color * opacity);

        for i in 0..(history.len() - 1) / RENDER_STRIDE {
            let next_tr = history[i * RENDER_STRIDE];
            let prev_tr = history[(i + 1) * RENDER_STRIDE];
            let Some(a) = reproject(&next_tr, *pos) else {
                continue;
            };
            let Some(b) = reproject(&prev_tr, *pos) else {
                continue;
            };
            gizmos.line(a, b, *color * opacity);
        }
    }
}
