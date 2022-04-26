use bevy::prelude::{OrthographicCameraBundle, UiCameraBundle, Commands};
use shared::player::MainCamera;



pub fn setup_player_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());
}

