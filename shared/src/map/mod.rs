mod loader;
mod render;
mod tiled_map;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use tiled_map::{
    texture::set_texture_filters_to_nearest,
    tiled::TiledMapPlugin,
};

use serde::Deserialize;

use crate::collider::*;
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


#[derive(Bundle)]
pub struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: MovementCollider,
    projectile_collider: ProjectileCollider,
    info: MapElementPosition,
}

#[derive(Component)]
pub struct Window {}

#[derive(Bundle)]
pub struct WindowBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    info: MapElementPosition,
    collider: MovementCollider,
    window: Window
}

#[derive(Component)]
pub struct MapElement {}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MapElementPosition {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: i32,
}

#[derive(Component, Default)]
pub struct ZombieSpawner {}

#[derive(Bundle)]
pub struct ZombieSpawnerBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    position: MapElementPosition,
    map_element: MapElement,
    spawner: ZombieSpawner,
}

impl ZombieSpawnerBundle {
    pub fn new(info: MapElementPosition) -> ZombieSpawnerBundle {
        ZombieSpawnerBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.10, 0.30, 0.50),
                    custom_size: Some(info.size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: info.position.extend(10.0),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            },
            position: info,
            map_element: MapElement {},
            spawner: ZombieSpawner {},
        }
    }
}

impl WindowBundle {
    pub fn new(info: MapElementPosition) -> WindowBundle {
        WindowBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.50, 0.50),
                    custom_size: Some(info.size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: info.position.extend(10.0),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            },
            collider: MovementCollider {},
            info,
            window: Window {},
        }
    }
}

impl WallBundle {
    pub fn new(info: MapElementPosition) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.25),
                    custom_size: Some(info.size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: info.position.extend(10.0),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            },
            collider: MovementCollider {},
            projectile_collider: ProjectileCollider {},
            info,
        }
    }
}

