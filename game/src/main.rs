#![feature(stmt_expr_attributes)]
#![feature(derive_default_enum)]

mod shared;
mod plugins;
mod ingameui;
mod character_animation;
mod menu;

mod p2p;

use bevy::{
    core::FixedTimestep, prelude::*, window::WindowDescriptor, ecs::schedule::ShouldRun
};

use bevy_ggrs::{SessionType, GGRSPlugin};
use bytemuck::{Pod, Zeroable};
use ggrs::{SessionBuilder, Config, P2PSession};
use shared::{
    game::{
        react_level_data, setup_zombie_game, system_zombie_game,
        GameState, ZombieGamePlugin, LevelMapRequested, system_unload_zombie_game, system_end_game, increase_frame_system, ZombieGame, GameSpeed,
    },
    zombies::zombie::{system_zombie_handle, Zombie, BotDestination, system_move_zombie},
    player::{input::{apply_input_players, FrameCount, input, BoxInput, AvailableGameController, system_gamepad_event, GGRSConfig, update_velocity_player, move_players}, interaction::system_interaction_player, system_unload_players, system_health_player, Player
    }, weapons::{weapons::{handle_weapon_input, Weapon, AmmunitionState, Projectile}, ammunition::{apply_velocity, movement_projectile}}, map::{render::system_unload_map, ZombieSpawner}, character::{Velocity, LookingAt, Death, CharacterMovementState}, health::{Health, HealthRegeneration}, collider::ProjectileCollider,
};
use shared::map::MapPlugin;
use crate::{
    plugins::{
        frame_cnt::FPSPlugin,
        web::WebPlugin,
    },
    character_animation::CharacterAnimationPlugin,
    menu::{
        homemenu::{HomeMenuPlugin, clear_home_menu, system_button_handle}, onlinemenu::OnlineMenuPlugin, 
    },
    ingameui::{
        ingameui::{system_clear_ingame_ui, system_weapon_ui, system_ingame_ui, setup_ingame_ui},
        player::{setup_player_camera, system_player_added}
    }, p2p::{config::P2PSystemLabel, checksum::{checksum_zombie, checksum_zombiegame}, online::system_cleanup_network_session}
};

use bevy_kira_audio::AudioPlugin;

const TIME_STEP: f32 = 1.0 / 60.0;

const ROLLBACK_DEFAULT: &str = "rollback_default";
const CHECKSUM_UPDATE: &str = "checksum_update";


fn print_events_system(mut session: ResMut<P2PSession<GGRSConfig>>) {
    for event in session.events() {
        println!("GGRS Event: {:?}", event);
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    
    let mut app = App::new();


    let game_speed = GameSpeed::default();


    GGRSPlugin::<GGRSConfig>::new()
        // define frequency of rollback game logic update
        .with_update_frequency(game_speed.1)
        // define system that returns inputs given a player handle, so GGRS can send the inputs around
        .with_input_system(input)
        // register types of components AND resources you want to be rolled back
        .register_rollback_type::<Player>()
        .register_rollback_type::<LookingAt>()
        .register_rollback_type::<Transform>()
        .register_rollback_type::<Velocity>()
        .register_rollback_type::<FrameCount>()
        .register_rollback_type::<AmmunitionState>()
        .register_rollback_type::<Projectile>()
        .register_rollback_type::<Zombie>()
        .register_rollback_type::<ZombieGame>()
        .register_rollback_type::<ZombieSpawner>()
        .register_rollback_type::<CharacterMovementState>()
        .register_rollback_type::<BotDestination>()
        .register_rollback_type::<Health>()
        .register_rollback_type::<ProjectileCollider>()
        //.register_rollback_type::<HealthRegeneration>()
        .register_rollback_type::<Death>()


        // these systems will be executed as part of the advance frame update
        .with_rollback_schedule(
            Schedule::default().with_stage(
                ROLLBACK_DEFAULT,
                SystemStage::parallel()
                    .with_system_set(
                        SystemSet::new()
                            .with_system(system_zombie_handle)
                            .with_system(apply_input_players)
                            .with_system(handle_weapon_input)
                            .with_system(system_interaction_player)
                            .label(P2PSystemLabel::Input)
                    )
                    .with_system_set(
                        SystemSet::new()
                            .with_system(update_velocity_player)
                            .with_system(system_move_zombie)
                            .label(P2PSystemLabel::Move)
                            .after(P2PSystemLabel::Input)
                    )
                    .with_system_set(
                        SystemSet::new()
                            .with_system(move_players)
                            .with_system(movement_projectile)
                            .with_system(apply_velocity)
                            .label(P2PSystemLabel::Collision)
                            .after(P2PSystemLabel::Move)
                    )
                    .with_system_set(
                        SystemSet::new()
                            .with_system(system_health_player)
                            .with_system(system_zombie_game)
                            .after(P2PSystemLabel::Collision)
                            .label(P2PSystemLabel::GameLogic)
                    )
                    .with_system_set(
                        SystemSet::new()
                            .with_system(increase_frame_system)
                            .with_system(system_end_game)
                            .after(P2PSystemLabel::GameLogic)
                    )
            )
            .with_stage_after(
                ROLLBACK_DEFAULT, 
                CHECKSUM_UPDATE,
                SystemStage::parallel()
                        .with_system(checksum_zombie)
                        .with_system(checksum_zombiegame)
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
    .insert_resource(LevelMapRequested{map: "maps/map_iso/iso_map.asset.ron".to_string(), level: "game/easy.level.ron".to_string()})
    .insert_resource(AvailableGameController{
        keyboard_mouse: true,
        gamepad: vec![]
    })
    .insert_resource(FrameCount { frame: 0 })
    .add_plugins(DefaultPlugins)
    .add_plugin(CharacterAnimationPlugin{ })
    .add_plugin(AudioPlugin{})
    .add_plugin(MapPlugin {});

    app.add_plugin(ZombieGamePlugin{});
    app.add_plugin(HomeMenuPlugin{});
    app.add_plugin(OnlineMenuPlugin{});

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
            .with_system(react_level_data)
            .with_system(system_player_added)
    )
    .add_system_set(
        SystemSet::on_exit(GameState::PlayingZombie)
            .with_system(system_unload_map)
            .with_system(system_clear_ingame_ui)
            .with_system(system_unload_players)
            .with_system(system_unload_zombie_game)
            .with_system(system_cleanup_network_session)
    );

    //if opts.benchmark_mode {
    //  app.add_plugin(FPSPlugin{});
    //}

    app.add_plugin(WebPlugin{});

    app.run();
}

