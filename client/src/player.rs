use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::TextureAtlas};
use shared::{player::{MainCamera, Player, LookingAt, AnimationTimer,CharacterMovementState}, utils::get_cursor_location, weapons::{weapons::WeaponState, loader::WeaponAssetState}};


pub fn setup_player_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());
}


