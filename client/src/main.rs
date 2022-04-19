mod config;
mod game;
mod map;
mod player;
mod plugins;

use bevy::asset::AssetServerSettings;
use bevy::{
    core::FixedTimestep, prelude::*, sprite::collide_aabb::collide, window::WindowDescriptor,
};
use bevy_ecs_tilemap::prelude::*;
use player::{Player, Projectile, Velocity};

use crate::game::{
    react_level_data, setup_zombie_game, system_zombie_game, system_zombie_handle, ZombieGame,
    ZombieGameState, ZombieLevelAsset, ZombieLevelAssetLoader, ZombieLevelAssetState,
};
use crate::map::{
    data::{MapElementPosition, MovementCollider, ProjectileCollider},
    MapPlugin,
};
use crate::player::{apply_velocity, input_player, movement_projectile, setup_players};
use crate::{
    game::{Game, GameState},
    plugins::frame_cnt::FPSPlugin,
};

use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin, Packet};

const TIME_STEP: f32 = 1.0 / 60.0;

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
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..WindowDescriptor::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(MapPlugin {});

    app.init_resource::<Game>()
        .init_resource::<ZombieGame>()
        .init_resource::<ZombieLevelAssetState>()
        .add_plugin(NetworkingPlugin::default())
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

    //if opts.benchmark_mode {
    //  app.add_plugin(FPSPlugin{});
    //}

    app.run();
}
