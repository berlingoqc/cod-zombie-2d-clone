use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_ecs_tilemap::prelude::*;

use shared::map::*;
use shared::collider::*;
use super::tiled_map::tiled::{TiledMap, TiledMapBundle};
use serde::Deserialize;

#[derive(Default)]
pub struct MapDataState {
    pub handle: Handle<MapDataAsset>,
    pub rendered: bool,
}

#[derive(Deserialize, Clone, Component)]
pub struct MapTiledData {
    pub path: String,
    pub transform: Vec3,
}

#[derive(Deserialize, TypeUuid, Clone, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct MapDataAsset {
    pub walls: Vec<MapElementPosition>,
    pub windows: Vec<MapElementPosition>,
    pub spawners: Vec<MapElementPosition>,
    pub tiled: MapTiledData,
}

impl MapDataAsset {
    pub fn render(&self, command: &mut Commands, asset_server: &AssetServer) {
        let handle: Handle<TiledMap> = asset_server.load(self.tiled.path.as_str());

        let map_entity = command.spawn().id();
        command
            .entity(map_entity)
            .insert(MapElement {})
            .insert_bundle(TiledMapBundle {
                tiled_map: handle,
                map: Map::new(0u16, map_entity),
                transform: Transform::from_xyz(
                    self.tiled.transform.x,
                    self.tiled.transform.y,
                    self.tiled.transform.z,
                ),
                ..Default::default()
            });

        for s in (&self.spawners).into_iter() {
            command
                .spawn()
                .insert_bundle(ZombieSpawnerBundle::new(s.clone()));
        }

        for w in (&self.walls).into_iter() {
            command
                .spawn()
                .insert(MapElement {})
                .insert_bundle(WallBundle::new(w.clone()));
        }

        for w in (&self.windows).into_iter() {
            command
                .spawn()
                .insert(MapElement {})
                .insert_bundle(WindowBundle::new(w.clone()));
        }
    }
}

pub fn render_map_data(
    commands: &mut Commands,
    map_data: &MapDataAsset,
    asset_server: &AssetServer,
) {
    map_data.render(commands, asset_server);
}

pub fn load_scene_system(
    asset_server: Res<AssetServer>,
    mut state: ResMut<MapDataState>,
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
    asset_server: Res<AssetServer>,
) {
    let data_asset = custom_assets.get(&state.handle);
    if state.rendered || data_asset.is_none() {
        return;
    }
    let map_data = data_asset.unwrap();

    render_map_data(&mut commands, &map_data, &asset_server);

    state.rendered = true;
}

pub fn react_event_scene(
    mut asset_events: EventReader<AssetEvent<MapDataAsset>>,
    mut commands: Commands,
    entity: Query<Entity, With<MapElement>>,
    mut state: ResMut<MapDataState>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Modified { .. } => {
                for element in entity.iter() {
                    commands.entity(element).despawn();
                }
                state.rendered = false;
            }
            _ => {}
        }
    }
}
