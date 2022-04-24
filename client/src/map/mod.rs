mod loader;
mod render;
mod tiled_map;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use tiled_map::{
    texture::set_texture_filters_to_nearest,
    tiled::TiledMapPlugin,
};

use shared::map::*;
use shared::collider::*;
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
