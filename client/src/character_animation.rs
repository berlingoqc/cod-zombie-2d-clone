
use bevy::{prelude::*, reflect::TypeUuid, asset::{LoadContext, AssetLoader, BoxedFuture, LoadedAsset}, utils::HashMap};
use serde::Deserialize;

use shared::{animation::{SpriteSheetAnimationsConfiguration, SpriteSheetConfiguration}, game::Zombie};
use shared::{player::{MainCamera, Player, LookingAt, AnimationTimer, CharacterMovementState}, utils::get_cursor_location, weapons::{weapons::WeaponState, loader::WeaponAssetState}};

#[derive(Deserialize, TypeUuid, Default, Component, Clone)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5011"]
pub struct CharacterAnimationConfiguration {
    pub animations: SpriteSheetAnimationsConfiguration,
    pub sprite_sheet: SpriteSheetConfiguration,
}

#[derive(Default)]
pub struct CharacterAnimationStateHandle {
    pub handle: Handle<CharacterAnimationConfiguration>,
    pub loaded: bool,
    pub config: Option<CharacterAnimationConfiguration>,
    pub texture_loaded: bool,
    pub handle_texture_atlas: Option<Handle<TextureAtlas>>,
}

#[derive(Default)]
pub struct CharacterAnimationConfigurationState(pub HashMap<String, CharacterAnimationStateHandle>);

impl CharacterAnimationConfigurationState {

	pub fn add_handler(&mut self, asset_server: &Res<AssetServer>, name: &str, path: &str) {
		self.0.insert(name.to_string(), CharacterAnimationStateHandle {
			handle: asset_server.load(path),
			loaded: false,
			config: None,
			texture_loaded: false,
            handle_texture_atlas: None
		});
	}
}

#[derive(Default)]
pub struct CharacterAnimationConfigurationLoader;

impl AssetLoader for CharacterAnimationConfigurationLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let map_data_asset = ron::de::from_bytes::<CharacterAnimationConfiguration>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(map_data_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animation.ron"]
    }
}

pub fn setup_character_animation_config(
    mut state: ResMut<CharacterAnimationConfigurationState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
	state.add_handler(&asset_server, "player", "characters/player/player.animation.ron");
	state.add_handler(&asset_server, "zombie", "characters/zombie/zombie.animation.ron")
}

pub fn system_character_animation_config(
    mut state: ResMut<CharacterAnimationConfigurationState>,
    custom_assets: ResMut<Assets<CharacterAnimationConfiguration>>
) {
	for state in state.0.values_mut() {
		if !state.loaded {
			let v = custom_assets.get(&state.handle);
			if v.is_some() {
				state.loaded = true;
	            state.config = Some(v.unwrap().clone());
			}
		}
	}
}

pub fn react_character_animation(
    mut asset_events: EventReader<AssetEvent<CharacterAnimationConfiguration>>,
    mut commands: Commands,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Modified { .. } => {
				println!("UPDATE");
            }
            _ => {}
        }
    }
}


fn validate_asset_loading(
    asset_server: &Res<AssetServer>,
	player_config_state: &mut CharacterAnimationStateHandle,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    if player_config_state.config.is_none() {
        return;
    } else if !player_config_state.texture_loaded {
        let config = player_config_state.config.as_ref().unwrap();
        let texture_handle = asset_server.load(config.sprite_sheet.path.as_str());
        let texture_atlas = TextureAtlas::from_grid(texture_handle, config.sprite_sheet.tile_size, config.sprite_sheet.columns, config.sprite_sheet.rows);
        player_config_state.handle_texture_atlas = Some(texture_atlases.add(texture_atlas));
        player_config_state.texture_loaded = true;
    }
}

pub fn system_animation_character(
    mut q_player: Query<(&CharacterMovementState, &mut TextureAtlasSprite, &mut AnimationTimer, &mut Handle<TextureAtlas>, &mut LookingAt, &mut Transform)>,

    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut player_config_state: ResMut<CharacterAnimationConfigurationState>,

    time: Res<Time>,
) {

    for (movement_state, mut atlas_sprite, mut timer, mut handle, mut looking_at, mut transform) in q_player.iter_mut() {

        timer.timer.tick(time.delta());

	    let mut player_config_state = player_config_state.0.get_mut(timer.asset_type.as_str()).unwrap();

        validate_asset_loading(&asset_server, &mut player_config_state, &mut texture_atlases);

        let config = player_config_state.config.as_ref();
        if config.is_none() { return; }
        let config = config.unwrap();

        let handle_texture_atlas = player_config_state.handle_texture_atlas.as_ref().unwrap();

        if handle.id != handle_texture_atlas.id {
            handle.id = handle_texture_atlas.id;
        }


        if let Some(animation) = config.animations.0.get(&movement_state.state) {
            let state = movement_state.sub_state.clone() + movement_state.state.as_str();
            if !timer.current_state.eq(state.as_str()) {
                timer.current_state = state.clone();
                timer.index = 0;
                timer.timer = Timer::from_seconds(animation.playback_speed, true);
            } else {
                if timer.timer.just_finished() {
                    timer.index += 1;
                    if timer.index >= animation.indexs.len() {
                        timer.index = 0;
                    }
                }
            }
            atlas_sprite.index = timer.offset + animation.indexs[timer.index];
        }
    }
}

pub fn system_looking_at(
    mut q_character: Query<(&mut Transform, &mut LookingAt)>
) {
    for (mut transform, mut looking_at) in q_character.iter_mut() {
        let diff = (looking_at.0 - transform.translation.truncate());
        let angle = diff.y.atan2(diff.x);

        transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}



pub struct CharacterAnimationPlugin {}

impl Plugin for CharacterAnimationPlugin {
	fn build(&self, app: &mut App) {
        app
			.init_resource::<CharacterAnimationConfigurationState>()
            .add_asset::<CharacterAnimationConfiguration>()
            .init_asset_loader::<CharacterAnimationConfigurationLoader>()
            .add_startup_system(setup_character_animation_config)
            .add_system_set(
                SystemSet::new()
                    .with_system(system_character_animation_config)
                    .with_system(system_animation_character)
                    .with_system(system_looking_at)
            )
			.add_system(react_character_animation);
	}
}

