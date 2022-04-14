mod plugins;
mod config;
mod map;
mod player;
mod game;


use bevy::{
    prelude::*,
    window::WindowDescriptor
};

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
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let mut moved = false;
    if keyboard_input.pressed(KeyCode::Up) {
        game.player.y += 1;
        moved = true;   
    }
    if keyboard_input.pressed(KeyCode::Down) {
        game.player.y -= 1;
        moved = true;   
    }
    if keyboard_input.pressed(KeyCode::Right) {
        game.player.x += 1;
        moved = true;   
    }
    if keyboard_input.pressed(KeyCode::Left) {
        game.player.x -= 1;
        moved = true;   
    }

    if moved {
        *transforms.get_mut(game.player.entity.unwrap()).unwrap() = Transform {
            translation: Vec3::new(
                game.player.x as f32,
                game.player.y as f32,
                6.,
            ),
            ..Transform::default()
        };
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
        ..SpriteBundle::default()
    };

    game.player.entity = Some(commands.spawn_bundle(sprite_bundle).id());

}
