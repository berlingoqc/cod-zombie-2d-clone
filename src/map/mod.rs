pub mod data;
pub mod loader;
pub mod render;

use std::io::Write;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, reflect::{TypeRegistry, TypeUuid}, asset::{AssetLoader, LoadContext, BoxedFuture, LoadedAsset}};

use bevy_ecs_tilemap::prelude::*;


use data::*;


fn render_map_data(
    commands: &mut Commands,
    map_data: &MapDataAsset,
) {
    map_data.render(commands);
}

pub fn load_scene_system(asset_server: Res<AssetServer>, mut state: ResMut<MapDataState>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut commands: Commands,
) {
    // Scenes are loaded just like any other asset.
    let handle: Handle<MapDataAsset> = asset_server.load("level_1.custom");
    state.handle = handle;
    state.rendered = false;

    asset_server.watch_for_changes().unwrap();
}

pub fn render_scene(
    mut state: ResMut<MapDataState>,
    custom_assets: ResMut<Assets<MapDataAsset>>,
    mut commands: Commands,
) {
    let data_asset = custom_assets.get(&state.handle);
    if state.rendered || data_asset.is_none() {
        return;
    }
    let map_data = data_asset.unwrap();

    render_map_data(&mut commands, &map_data);

    state.rendered = true;
}


pub fn react_event_scene(
    mut asset_events: EventReader<AssetEvent<MapDataAsset>>,
    custom_assets: ResMut<Assets<MapDataAsset>>,
    mut commands: Commands,
    // TODO change for a parent entity for the whole map
    mut entity: Query<Entity, With<MapElementPosition>>,
    mut state: ResMut<MapDataState>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Modified { handle } => {
                for element in entity.iter() {
                    commands.entity(element).despawn();
                }
                state.rendered = false;
            },
            _ => {}
        }
    }
}
