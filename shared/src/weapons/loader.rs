use std::rc::Weak;

use bevy::{prelude::*, utils::HashMap, asset::{AssetLoader, LoadContext, BoxedFuture, LoadedAsset}, reflect::TypeUuid};
use serde::Deserialize;

use super::weapons::Weapon;


#[derive(Deserialize, TypeUuid, Default, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5021"]
pub struct WeaponsAsset {
	pub weapons: Vec<Weapon>
}


#[derive(Default)]
pub struct WeaponAssetState {
    pub handle: Handle<WeaponsAsset>,
    pub loaded: bool,
    pub weapons: Vec<Weapon>
}

#[derive(Default)]
pub struct WeaponAssetLoader;

impl AssetLoader for WeaponAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let map_data_asset = ron::de::from_bytes::<WeaponsAsset>(bytes)?;
            println!("LOADED");
            load_context.set_default_asset(LoadedAsset::new(map_data_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub fn setup_weapons_asset(
    mut state: ResMut<WeaponAssetState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<WeaponsAsset> = asset_server.load("weapons/list.ron");
    state.handle = handle;
    state.loaded = false;
}

pub fn system_weapon_asset(
    mut state: ResMut<WeaponAssetState>,
    custom_assets: ResMut<Assets<WeaponsAsset>>
) {
	if !state.loaded {
		let v = custom_assets.get(&state.handle);
		if v.is_some() {
			state.loaded = true;
            state.weapons = v.unwrap().weapons.iter().map(|x| x.clone()).collect()
		}
	}
}


pub struct WeaponAssetPlugin {}

impl Plugin for WeaponAssetPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<WeaponAssetState>()
			.add_asset::<WeaponsAsset>()
			.init_asset_loader::<WeaponAssetLoader>()

            .add_startup_system(setup_weapons_asset)
            .add_system(system_weapon_asset);
	}
}