pub mod data;

mod loader;
mod render;
mod tiled_map;

use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{TypeRegistry, TypeUuid},
    sprite::MaterialMesh2dBundle,
};
use bevy_ecs_tilemap::prelude::*;

use tiled_map::{
    texture::set_texture_filters_to_nearest,
    tiled::{MapData, TiledMapBundle, TiledMapPlugin},
    tiled_usage::startup_tiled,
};

use data::*;
use loader::*;
use render::*;

pub struct MapPlugin {}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MapElementPosition>()
            .add_plugin(TilemapPlugin)
            .add_plugin(TiledMapPlugin)
            .init_resource::<MapDataState>()
            .add_asset::<MapDataAsset>()
            .init_asset_loader::<MapDataAssetLoader>()
            .add_startup_system(load_scene_system)
            .add_system(react_event_scene)
            .add_system(render_scene)
            .add_system(set_texture_filters_to_nearest);
    }
}
