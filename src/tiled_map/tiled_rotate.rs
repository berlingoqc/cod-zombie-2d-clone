use crate::tiled::*;
use bevy::{math::Vec3, prelude::*};
use bevy_ecs_tilemap::prelude::*;

#[path = "../helpers/mod.rs"]
mod helpers;
mod tiled;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // `rotate.tmx` is generated by Tiled (map editor).
    // The map consists of a single tile, flipped and rotated in various ways.
    // The map refers to `dungeon.png` (created in Piskel).
    let handle: Handle<TiledMap> = asset_server.load("rotate.tmx");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform {
            translation: Vec3::new(0., 0., 1.0),
            scale: Vec3::new(4.0, 4.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Tiled map editor example with flipping and rotation."),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
