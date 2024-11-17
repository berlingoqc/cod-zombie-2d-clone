use bevy::prelude::*;

use crate::shared::player::MainCamera;

pub fn get_cursor_location(
    q_windows: &Query<&Window>,
    q_camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to
    let wnd = q_windows.single();

    if let Some(world_position) = wnd.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        return world_position;
    } else {
        return Vec2::new(0., 0.);
    }
}


pub fn vec2_perpendicular_clockwise(vec: Vec2) -> Vec2 {
    Vec2::new(vec.y, -vec.x)
}

pub fn vec2_perpendicular_counter_clockwise(vec: Vec2) -> Vec2 {
    Vec2::new(-vec.y, vec.x)
}

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct Checksum {
    pub value: u16,
}

/// Computes the fletcher16 checksum, copied from wikipedia: <https://en.wikipedia.org/wiki/Fletcher%27s_checksum>
pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;

    for byte in data {
        sum1 = (sum1 + *byte as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    (sum2 << 8) | sum1
}