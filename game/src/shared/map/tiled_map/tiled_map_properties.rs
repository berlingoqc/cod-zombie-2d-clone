use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_ecs_tilemap::{TilePos, Tile, Map};
use tiled::PropertyValue;

use super::tiled::{MapRenderedEvent, TiledMap};
use crate::shared::{collider::{MovementCollider, ProjectileCollider}, map::{ZombieSpawnerBundle, MapElementPosition, MapElement, WindowBundle, render::MapDataState}, game::ZombieGamePanelEvent};

pub fn system_react_tiled_for_properties(
    mut commands: Commands,
    mut ev_maprenderd: EventReader<MapRenderedEvent>,

    mut ev_panel_event: EventWriter<ZombieGamePanelEvent>,

    q_tiled: Query<(Entity, &Tile, &TilePos)>,

    maps: Res<Assets<TiledMap>>,
    query: Query<(Entity, &Handle<TiledMap>, &Transform, &mut Map)>,

    mut map_state: ResMut<MapDataState>,
) {
    // TODO need to do the same when new tiled is added if i ever need it
    let mut windows: Vec<Vec<Vec2>> = vec![];
    for e in ev_maprenderd.iter() {
    if let Ok((_entity, _, transform, _map)) = query.get_single() {
            if let Some(map) = maps.get(&e.handle) {
                for (e, tile, pos) in q_tiled.iter() {
                    for tileset in map.map.tilesets() {
                        if let Some(tile_data) = tileset.get_tile(tile.texture_index.into()) {
                            for (k, v) in tile_data.properties.iter() {
                                if k.eq("collider") {
                                    match v {
                                        PropertyValue::BoolValue(b) => {
                                            if !b {
                                                continue;
                                            }
                                            let position = (transform.translation
                                                + Vec3::new(32., 32., 0.)
                                                    * Vec3::new(pos.0 as f32, pos.1 as f32, 0.))
                                                + Vec3::new(16., 16., 0.);
                                            commands
                                                .spawn()
                                                .insert(ProjectileCollider {})
                                                .insert(MovementCollider {
                                                    size: Vec2::new(32., 32.),
                                                    allowed_entity_type: vec![],
                                                })
                                                .insert(Transform {
                                                    translation: position,
                                                    ..default()
                                                });
                                        }
                                        _ => {
                                            info!("IGNORING");
                                        }
                                    }
                                }
                                if k.eq("zombie_spawn") {
                                    let position = (transform.translation
                                        + Vec3::new(32., 32., 0.)
                                            * Vec3::new(pos.0 as f32, pos.1 as f32, 0.))
                                        + Vec3::new(16., 16., 0.);
                                    commands.spawn().insert_bundle(ZombieSpawnerBundle::new(MapElementPosition {
                                        position: position.truncate(),
                                        ..Default::default()
                                    })).insert(MapElement{});
                                }
                                if k.eq("window") {
                                    let position = ((transform.translation
                                        + Vec3::new(32., 32., 0.)
                                            * Vec3::new(pos.0 as f32, pos.1 as f32, 0.))
                                        + Vec3::new(16., 16., 0.)).truncate();
                                    let mut found_match = false;
                                    'outer: for window_group in windows.iter_mut() {
                                        for window in window_group.iter() {
                                            if let Some(_) = collide(window.extend(0.), Vec2::new(35., 35.), position.extend(0.), Vec2::new(35., 35.)) {
                                                window_group.push(position);
                                                found_match = true;
                                                break 'outer;
                                            }
                                        }
                                    }
                                    if !found_match {
                                        windows.push(vec![position]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("WINDOWS GROUPS {:?}", windows);
        // Crée la size du bloc
        for window_group in windows.iter() {
            // TODO shit code
            let w1 = window_group.get(0).unwrap();
            let w2 = window_group.get(1).unwrap();
            let (size, position) = if w1.x == w2.x { (Vec2::new(32., 64.), Vec2::new(0., -16.)) } else { (Vec2::new(64., 32.), Vec2::new(-16., 0.))};

            commands
                .spawn()
                .insert(MapElement {})
                .insert_bundle(WindowBundle::new(MapElementPosition{
                    position: (*w1) - position,
                    size: size,
                    ..default()
            }));
        }

        map_state.rendered_map_objects = true;
    }
}