#![feature(stmt_expr_attributes)]


mod config;
mod plugins;
mod client;
mod ingameui;
mod player;
mod character_animation;

use bevy::{
    core::FixedTimestep, prelude::*, window::WindowDescriptor,
};

use shared::{
    game::{
        react_level_data, setup_zombie_game, system_zombie_game, system_zombie_handle, ZombieGame,
        Game, GameState,
        ZombieGameState, ZombieLevelAsset, ZombieLevelAssetLoader, ZombieLevelAssetState, ZombieGamePlugin, LevelMapRequested,
    },
    player::{
        apply_velocity, input_player, movement_projectile, setup_players, system_interaction_player
    }, weapons::weapons::{handle_weapon_input},
};
use shared::map::MapPlugin;
use crate::{plugins::frame_cnt::FPSPlugin, character_animation::CharacterAnimationPlugin};

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let opts = config::Opts::get();
    info!("opts: {:?}", opts);

    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Zombie".to_string(),
        width: 500.,
        height: 300.,
        resizable: true,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .insert_resource(LevelMapRequested{map: opts.map, level: opts.level})
    .add_plugins(DefaultPlugins)
    .add_plugin(CharacterAnimationPlugin{ })
    .add_plugin(MapPlugin {});

    app.add_plugin(ZombieGamePlugin{});

    if opts.host == "" {
        info!("Startin single player mode");
        app
        .add_system_set(
            SystemSet::on_enter(GameState::PlayingZombie)
                .with_system(player::setup_player_camera)
                .with_system(ingameui::setup_ingame_ui)
                .with_system(setup_zombie_game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::PlayingZombie)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(ingameui::system_ingame_ui)
                .with_system(ingameui::system_weapon_ui)
                .with_system(system_zombie_handle)
                .with_system(system_zombie_game)
                .with_system(apply_velocity)
                .with_system(input_player)
                .with_system(system_interaction_player)
                .with_system(handle_weapon_input)
                .with_system(movement_projectile)
                .with_system(react_level_data)
        );
    } else {
        info!("Startin multiplayer mode");
        app.add_plugin(client::NetworkClientPlugin{});
    }

    if opts.benchmark_mode {
      app.add_plugin(FPSPlugin{});
    }

    #[cfg(target_arch = "wasm32")]
    app.add_system(update_window_size);

    app.run();
}


#[cfg(target_arch = "wasm32")]
fn update_window_size(mut windows: ResMut<Windows>) {
    //See: https://github.com/rust-windowing/winit/issues/1491
    // TODO: use window resize event instead of polling
    use approx::relative_eq;
    let web_window = web_sys::window().unwrap();
    let width = web_window.inner_width().unwrap().as_f64().unwrap() as f32;
    let height = web_window.inner_height().unwrap().as_f64().unwrap() as f32;

    let window = windows.get_primary_mut().unwrap();
    if relative_eq!(width, window.width()) && relative_eq!(height, window.height()) {
        return;
    }

    info!(
        "resizing canvas {:?}, old size {:?}",
        (width, height),
        (window.width(), window.height())
    );
    window.set_resolution(width, height);
}
