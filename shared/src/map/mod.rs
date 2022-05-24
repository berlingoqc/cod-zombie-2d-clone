mod loader;
mod tiled_map;
pub mod render;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use tiled_map::{
    texture::set_texture_filters_to_nearest,
    tiled::TiledMapPlugin,
};

use serde::Deserialize;

use crate::{collider::*, health::Health, game::GameState, player::interaction::{PlayerInteraction, PlayerInteractionType}};
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

            .add_system_set(
                SystemSet::on_enter(GameState::PlayingZombie)
                    .with_system(load_scene_system)
            )
            .add_system_set(
                SystemSet::on_update(GameState::PlayingZombie)
                    .with_system(react_event_scene)
                    .with_system(render_scene)
                    .with_system(set_texture_filters_to_nearest)
            );
   }
}



#[derive(Component, Default)]
pub struct Size(pub Vec2);

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
    interaction: PlayerInteraction,
}

#[derive(Component)]
pub struct WindowPanel {}

#[derive(Bundle)]
pub struct WindowPanelBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    panel: WindowPanel,
    health: Health,
    size: Size,
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
            collider: MovementCollider {
                size: info.size,
            },
            info,
            window: Window {},
            interaction: PlayerInteraction {
                interaction_available: true,
                interaction_type: PlayerInteractionType::RepairWindow,
                interaction_size: Vec2::new(150., 150.),
                interaction_timeout: 1.2
            }
        }
    }
}

impl WindowPanelBundle {
    pub fn new(parent: MapElementPosition, health: f32,  index: u32, offset: f32) -> WindowPanelBundle {
        // need to find the direction vector of the window
        let (direction, size)  = if parent.size.x > parent.size.y { // horizontal
            (Vec2::new(1., 0.), Vec2::new(10., 20.))
        } else { (Vec2::new(0., 1.), Vec2::new(20., 10.)) }; // vertical
        // create a shap to fit the direction of the vector
        // spawn it a the location with the index and offset
        let scale = if index % 2 == 0 { 1. } else { -1. };
        let translation = direction * (scale * offset * (index as f32 / 2.).ceil());
        WindowPanelBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(size),
                    ..Sprite::default()
                },
                transform: Transform {
                    translation: translation.extend(10.),
                    ..Transform::default()
                },
                ..SpriteBundle::default()
            },
            //collider: MovementCollider {
            //    size: Vec2::new(0., 0.)
            //},
            panel: WindowPanel {},
            size: Size(size),
            health: Health { current_health: health, max_health: health }
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
            collider: MovementCollider {
                size: info.size,
            },
            projectile_collider: ProjectileCollider {},
            info,
        }
    }
}

