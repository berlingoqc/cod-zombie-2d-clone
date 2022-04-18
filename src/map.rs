
use std::io::Write;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, reflect::{TypeRegistry, TypeUuid}, asset::{AssetLoader, LoadContext, BoxedFuture, LoadedAsset}};

use bevy_ecs_tilemap::prelude::*;

use serde::Deserialize;

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct Collider {}

#[derive(Component)]
pub struct CollisionEvent {}

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
    info: MapElementPosition
}

#[derive(Component, Reflect, Default, Deserialize, Clone)]
#[reflect(Component)]
pub struct MapElementPosition {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: i32
}


#[derive(Default)]
pub struct MapDataState {
    handle: Handle<MapDataAsset>,
    rendered: bool,
}

#[derive(Deserialize, TypeUuid, Clone, Component)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct MapDataAsset {
    pub walls: Vec<MapElementPosition>,
}

impl MapDataAsset {
    pub fn render(&self, command: &mut Commands) {
        for w in (&self.walls).into_iter() {
            command.spawn_bundle(WallBundle::new(w.clone()));
        }
    }
}

#[derive(Default)]
pub struct MapDataAssetLoader;

impl AssetLoader for MapDataAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            println!("Importing data");
            let map_data_asset = ron::de::from_bytes::<MapDataAsset>(bytes)?;

            //let mut world = World::default();
            //world.insert_resource::<MapDataAsset>(map_data_asset);
            //world.spawn().insert(MapPipeline{});
            //load_context.set_default_asset(LoadedAsset::new(Scene::new(world)));
            load_context.set_default_asset(LoadedAsset::new(map_data_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

impl WallBundle {
    fn new(info: MapElementPosition) -> WallBundle {
        WallBundle{
           sprite_bundle: SpriteBundle{
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
           collider: Collider{},
           info
        }
    }
}


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
