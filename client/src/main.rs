#![feature(stmt_expr_attributes)]
#![feature(derive_default_enum)]

mod config;
mod plugins;
mod client;
mod ingameui;
mod homemenu;
mod player;
mod character_animation;
mod ui_utils;
mod localmultiplayerui;

use bevy::{
    core::FixedTimestep, prelude::*, window::WindowDescriptor, ecs::schedule::ShouldRun
};

use shared::{
    game::{
        react_level_data, setup_zombie_game, system_zombie_game,
        GameState, ZombieGamePlugin, LevelMapRequested, system_unload_zombie_game, system_end_game,
    },
    zombies::zombie::system_zombie_handle,
    player::{input::{input_player, self}, interaction::system_interaction_player, system_unload_players, system_health_player
    }, weapons::{weapons::{handle_weapon_input}, ammunition::{apply_velocity, movement_projectile}}, map::render::system_unload_map,
};
use shared::map::MapPlugin;
use crate::{
    plugins::{
        frame_cnt::FPSPlugin,
        web::WebPlugin,
    },
    character_animation::CharacterAnimationPlugin,
    homemenu::HomeMenuPlugin, ingameui::system_clear_ingame_ui, player::system_player_added
};
use bevy_kira_audio::AudioPlugin;

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
    .insert_resource(input::AvailableGameController{
        keyboard_mouse: true,
        gamepad: vec![]
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(CharacterAnimationPlugin{ })
    .add_plugin(AudioPlugin{})
    .add_plugin(MapPlugin {});

    app.add_plugin(ZombieGamePlugin{});
    app.add_plugin(HomeMenuPlugin{});

    app.add_system(input::system_gamepad_event);

    app.add_startup_system(player::setup_player_camera);

    if opts.host == "" {
        info!("Startin single player mode");
        app
        .add_system_set(
            SystemSet::on_enter(GameState::PlayingZombie)
                .with_system(setup_zombie_game)
                .with_system(ingameui::setup_ingame_ui),
        )
        .add_system_set(
            SystemSet::on_update(GameState::PlayingZombie)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64).chain(
                        (|In(input): In<ShouldRun>, state: Res<State<GameState>>| {
                            if state.current() == &GameState::PlayingZombie {
                                input
                            } else {
                                ShouldRun::No
                            }
                        })
                ))
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
                .with_system(system_player_added)
                .with_system(system_health_player)
                .with_system(system_end_game)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::PlayingZombie)
                .with_system(system_unload_map)
                .with_system(system_clear_ingame_ui)
                .with_system(system_unload_players)
                .with_system(system_unload_zombie_game)
        );
    } else {
        info!("Startin multiplayer mode");
        app.add_plugin(client::NetworkClientPlugin{});
    }

    if opts.benchmark_mode {
      app.add_plugin(FPSPlugin{});
    }

    app.add_plugin(WebPlugin{});

    app.run();
}

