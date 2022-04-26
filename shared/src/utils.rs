use bevy::prelude::*;

use crate::player::MainCamera;

pub fn get_cursor_location(
    wnds: &Windows,
    q_camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    let window = match camera.target {
        bevy::render::camera::RenderTarget::Window(w) => w,
        _ => panic!("camera not rendering to windows"),
    };

    // get the window that the camera is displaying to
    let wnd = wnds.get(window).unwrap();

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        return world_pos;
    } else {
        return Vec2::new(0., 0.);
    }
}