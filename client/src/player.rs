use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::TextureAtlas};
use shared::{player::{MainCamera, Player, LookingDirection, AnimationTimer}, utils::get_cursor_location};


pub fn setup_player_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn().insert_bundle(UiCameraBundle::default());
}


#[derive(Component, Default)]
pub struct CharacterTextureHandle {
    pub handle: Handle<TextureAtlas>,
}

pub fn setup_character_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut character_texture_handle: ResMut<CharacterTextureHandle>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("character_spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(112.0, 112.0), 8, 6);
    character_texture_handle.handle = texture_atlases.add(texture_atlas);
}

pub fn system_animation(
    mut q_player: Query<(&mut TextureAtlasSprite, &mut AnimationTimer, &mut Handle<TextureAtlas>, &mut LookingDirection, &GlobalTransform, &mut Transform), With<Player>>,

    mut character_texture_handle: ResMut<CharacterTextureHandle>,

    time: Res<Time>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {

    for (mut atlas_sprite, mut timer, mut handle, mut looking_direction, global_transform, mut transform) in q_player.iter_mut() {
        if handle.id != character_texture_handle.handle.id {
            handle.id = character_texture_handle.handle.id;
            atlas_sprite.index = 7
        }

        timer.tick(time.delta());
        
        if timer.just_finished() {
            if atlas_sprite.index == 7 {
                atlas_sprite.index = 0;
            } else {
                atlas_sprite.index += 1;
            }
        }

        let mouse_location = get_cursor_location(&wnds, &q_camera);

        let diff = (mouse_location - transform.translation.truncate());
        let angle = diff.y.atan2(diff.x);

        transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);

        transform.rotation = Quat::from_rotation_z(angle);
    }

}
