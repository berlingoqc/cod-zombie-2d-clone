mod plugins;
mod config;
mod map;
mod player;
mod game;


use bevy::{
    prelude::*,
    window::WindowDescriptor, sprite::collide_aabb::collide
};
use player::Player;
use map::{Collider, MapElementPosition};

use crate::{plugins::frame_cnt::FPSPlugin, map::setup_map, game::{Game, GameState}};

fn main() {
    let opts = config::Opts::get();
    info!("opts: {:?}", opts);

    let vsync = opts.fps == 60 && !opts.benchmark_mode;

    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Zombie".to_string(),
        width: 500.,
        height: 300.,
        resizable: true,
        vsync,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup_camera)
    .add_startup_system(setup_map);

    app
        .init_resource::<Game>()
        .add_state(GameState::Playing)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player)
        );
    

    //if opts.benchmark_mode {
        app.add_plugin(FPSPlugin{});
    //}

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    collider_query: Query<(Entity, &Transform, &MapElementPosition), (With<Collider>, With<MapElementPosition>, Without<Player>)>,
) {
    let mut player_transform = query.single_mut();

    let mut movement = Vec3::default();
    let mut moved = false;

    if keyboard_input.pressed(KeyCode::Up) {
        movement += Vec3::new(0.,1.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        movement += Vec3::new(0.,-1.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Left) {
        movement += Vec3::new(-1.,0.,0.);
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        movement += Vec3::new(1.,0.,0.);
        moved = true;
    }

    if !moved { return; }

    let dest = player_transform.translation + movement;

    let mut save_move = true;
    for (collider_entity, transform, info) in collider_query.iter() {
        let collision = collide(
            dest,
            Vec2::new(25.,25.),
            transform.translation,
            info.size
        );
        if let Some(collision) = collision {
            save_move = false;
        }
    }

    if save_move {
        player_transform.translation = dest;
    }
}

fn setup(
    mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>
) {
    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..Sprite::default()
        },
        transform: Transform {
           translation: Vec3::new(0.,0.,10.),
           ..Transform::default()
        },
        ..SpriteBundle::default()
    };

    commands
        .spawn()
        .insert(Player{})
        .insert_bundle(sprite_bundle);

}
