use std::io::Write;

use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{TypeRegistry, TypeUuid},
    sprite::MaterialMesh2dBundle,
};

use bevy_ecs_tilemap::prelude::*;

use serde::Deserialize;

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MovementCollider {}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct ProjectileCollider {}

#[derive(Component)]
pub struct CollisionEvent {}

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
    window: Window,
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

#[derive(Deserialize, TypeUuid, Clone, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct MapDataAsset {
    pub walls: Vec<MapElementPosition>,
    pub windows: Vec<MapElementPosition>,
    pub spawners: Vec<MapElementPosition>,
    pub tiled: MapTiledData,
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
