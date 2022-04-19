use super::tiled::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn startup_tiled(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<TiledMap> = asset_server.load("map.tmx");
    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform::from_xyz(-700., -500., 0.0),
        ..Default::default()
    });
}
