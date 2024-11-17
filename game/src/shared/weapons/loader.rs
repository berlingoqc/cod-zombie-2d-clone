use std::rc::Weak;

use bevy::{prelude::*, utils::HashMap, asset::{AssetLoader, LoadContext, LoadedAsset}};
use serde::Deserialize;

use crate::shared::asset_error::BlobAssetLoaderError;

use super::weapons::{Weapon, AmmunitionState, WeaponState};


#[derive(Deserialize, Asset, TypePath, Default, Component)]
pub struct WeaponsAsset {
	pub weapons: Vec<Weapon>
}


#[derive(Default, Resource)]
pub struct WeaponAssetState {
    pub handle: Handle<WeaponsAsset>,
    pub loaded: bool,
    pub weapons: Vec<Weapon>
}

#[derive(Default)]
pub struct WeaponAssetLoader;

impl AssetLoader for WeaponAssetLoader {
    type Asset = WeaponsAsset;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let map_data_asset = ron::de::from_bytes::<WeaponsAsset>(&bytes).unwrap();
            Ok(map_data_asset)
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
    let handle: Handle<WeaponsAsset> = asset_server.load("weapons/weapons.ron");
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
			.init_asset::<WeaponsAsset>()
			.init_asset_loader::<WeaponAssetLoader>()
            .add_systems(Startup, setup_weapons_asset)
            .add_systems(Update, system_weapon_asset)
            .add_systems(Update, react_weapon_asset_change);
	}
}

pub fn react_weapon_asset_change(
    mut asset_events: EventReader<AssetEvent<WeaponsAsset>>,
    custom_assets: ResMut<Assets<WeaponsAsset>>,
	mut query_player_weapon: Query<(&mut AmmunitionState, &mut WeaponState, &mut Weapon, &Parent), With<WeaponState>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Modified {  id } => {
                let asset = custom_assets.get(id.clone()).unwrap();
                for (mut ammo_state, _, mut weapon, parent) in query_player_weapon.iter_mut() {
                    let new_config = asset.weapons.iter().find(|&x| x.name.eq(weapon.name.as_str()));
                    if let Some(new_config) = new_config {
                        weapon.automatic = new_config.automatic;
                        weapon.ammunition = new_config.ammunition.clone();
                        weapon.asset_name = new_config.asset_name.clone();
                        weapon.firing_rate = new_config.firing_rate;
                        weapon.offset = new_config.offset;
                        weapon.reloading_time = new_config.reloading_time;

                        ammo_state.remaining_ammunition = new_config.ammunition.magasin_nbr_starting * new_config.ammunition.magasin_size;
                    }
                }
            },
            _ => {}
        }
    }

}
