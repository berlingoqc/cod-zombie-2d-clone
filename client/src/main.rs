#![feature(stmt_expr_attributes)]
#![feature(derive_default_enum)]

mod config;
mod plugins;
mod ingameui;
mod character_animation;
mod menu;

use std::net::SocketAddr;

use bevy::{
    core::FixedTimestep, prelude::*, window::WindowDescriptor, ecs::schedule::ShouldRun
};

use bevy_ggrs::{SessionType, GGRSPlugin};
use bytemuck::{Pod, Zeroable};
use ggrs::{SessionBuilder, UdpNonBlockingSocket, Config};
use shared::{
    game::{
        react_level_data, setup_zombie_game, system_zombie_game,
        GameState, ZombieGamePlugin, LevelMapRequested, system_unload_zombie_game, system_end_game, increase_frame_system,
    },
    zombies::zombie::system_zombie_handle,
    player::{input::{input_player, FrameCount, input, BoxInput, AvailableGameController, system_gamepad_event, GGRSConfig}, interaction::system_interaction_player, system_unload_players, system_health_player
    }, weapons::{weapons::{handle_weapon_input}, ammunition::{apply_velocity, movement_projectile}}, map::render::system_unload_map, character::Velocity,
};
use shared::map::MapPlugin;
use crate::{
    plugins::{
        frame_cnt::FPSPlugin,
        web::WebPlugin,
    },
    character_animation::CharacterAnimationPlugin,
    menu::{
        homemenu::{HomeMenuPlugin, clear_home_menu, system_button_handle}, 
    },
    ingameui::{
        ingameui::{system_clear_ingame_ui, system_weapon_ui, system_ingame_ui, setup_ingame_ui},
        player::{setup_player_camera, system_player_added}
    }
};

use bevy_kira_audio::AudioPlugin;

const TIME_STEP: f32 = 1.0 / 60.0;

const ROLLBACK_DEFAULT: &str = "rollback_default";



fn main() {
    let opts = config::Opts::get();
    info!("opts: {:?}", opts);

    let mut app = App::new();

    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(2)
        .with_max_prediction_window(12)
        .with_input_delay(2);

    sess_build = sess_build.add_player(ggrs::PlayerType::Local, 0).unwrap();
    
    let remote_addr: SocketAddr = opts.host.parse().unwrap();
    sess_build = sess_build.add_player(ggrs::PlayerType::Remote(remote_addr), 1).unwrap();

    let socket = UdpNonBlockingSocket::bind_to_port(opts.port as u16).unwrap();
    let sess = sess_build.start_p2p_session(socket).unwrap();

    GGRSPlugin::<GGRSConfig>::new()
        // define frequency of rollback game logic update
        .with_update_frequency(60)
        // define system that returns inputs given a player handle, so GGRS can send the inputs around
        .with_input_system(input)
        // register types of components AND resources you want to be rolled back
        .register_rollback_type::<Transform>()
        //.register_rollback_type::<Velocity>()
        .register_rollback_type::<FrameCount>()
        // these systems will be executed as part of the advance frame update
        .with_rollback_schedule(
            Schedule::default().with_stage(
                ROLLBACK_DEFAULT,
                SystemStage::parallel()
                    .with_system(increase_frame_system),
            ),
        )
        // make it happen in the bevy app
        .build(&mut app);

    // Create an GGRS session
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
    .insert_resource(AvailableGameController{
        keyboard_mouse: true,
        gamepad: vec![]
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(CharacterAnimationPlugin{ })
    .add_plugin(AudioPlugin{})
    .add_plugin(MapPlugin {});


    app.insert_resource(sess)
        .insert_resource(SessionType::P2PSession)
        // register a resource that will be rolled back
        .insert_resource(FrameCount { frame: 0 });

    app.add_plugin(ZombieGamePlugin{});
    app.add_plugin(HomeMenuPlugin{});

    app.add_system(system_gamepad_event);

    app.add_startup_system(setup_player_camera);

    app
    .add_system_set(
        SystemSet::on_enter(GameState::PlayingZombie)
            .with_system(setup_zombie_game)
            .with_system(setup_ingame_ui)
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
            .with_system(system_ingame_ui)
            .with_system(system_weapon_ui)
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

    if opts.benchmark_mode {
      app.add_plugin(FPSPlugin{});
    }

    app.add_plugin(WebPlugin{});

    app.run();
}

