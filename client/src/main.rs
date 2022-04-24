mod config;
mod map;
mod plugins;
mod client;

use bevy::{
    core::FixedTimestep, prelude::*, window::WindowDescriptor,
};

use shared::{
    game::{
        react_level_data, setup_zombie_game, system_zombie_game, system_zombie_handle, ZombieGame,
        Game, GameState,
        ZombieGameState, ZombieLevelAsset, ZombieLevelAssetLoader, ZombieLevelAssetState,
    },
    player::{
        apply_velocity, input_player, movement_projectile, setup_players
    },
};
use crate::map::MapPlugin;
use crate::plugins::frame_cnt::FPSPlugin;

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
    //.insert_resource(opts)
    .add_plugins(DefaultPlugins)
    .add_plugin(client::NetworkClientPlugin{})
    .add_plugin(MapPlugin {});

    app.init_resource::<Game>()
        .init_resource::<ZombieGame>()
        .init_resource::<ZombieLevelAssetState>()
        .add_asset::<ZombieLevelAsset>()
        .init_asset_loader::<ZombieLevelAssetLoader>()
        .add_state(GameState::PlayingZombie)
        .add_state(ZombieGameState::Starting)
        .add_system_set(
            SystemSet::on_enter(GameState::PlayingZombie)
                .with_system(setup_players)
                .with_system(setup_zombie_game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::PlayingZombie)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(system_zombie_handle)
                .with_system(system_zombie_game)
                .with_system(apply_velocity)
                .with_system(input_player)
                .with_system(movement_projectile)
                .with_system(react_level_data),
        );

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
