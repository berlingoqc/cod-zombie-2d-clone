#![feature(stmt_expr_attributes)]
#![feature(derive_default_enum)]

mod shared;
mod plugins;
mod ingameui;
mod character_animation;
mod menu;

mod p2p;

use bevy::{
    prelude::*
};

use bevy_ggrs::*;
use ggrs::{SessionBuilder, Config, P2PSession};
use shared::{
    character::{CharacterMovementState, Death, LookingAt, Velocity}, collider::ProjectileCollider, game::{
        increase_frame_system, react_level_data, setup_zombie_game, system_end_game, system_unload_zombie_game, system_zombie_game, GameSpeed, GameState, LevelMapRequested, ZombieGame, ZombieGamePlugin
    }, health::{Health, HealthRegeneration}, map::{render::system_unload_map, ZombieSpawner}, player::{input::{self, apply_input_players, input, move_players, system_gamepad_event, update_velocity_player, AvailableGameController, BoxInput, FrameCount, GGRSConfig}, interaction::system_interaction_player, system_health_player, system_unload_players, Player
    }, weapons::{ammunition::{apply_velocity, movement_projectile}, weapons::{handle_weapon_input, AmmunitionState, Projectile, Weapon}}, zombies::zombie::{system_move_zombie, system_zombie_handle, BotDestination, Zombie}
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
    }, p2p::{config::P2PSystemLabel, checksum::{checksum_zombie, checksum_zombiegame}}
};

use bevy_kira_audio::AudioPlugin;

const TIME_STEP: f32 = 1.0 / 60.0;

const ROLLBACK_DEFAULT: &str = "rollback_default";
const CHECKSUM_UPDATE: &str = "checksum_update";

pub type BoxConfig = GgrsConfig<BoxInput>;


/*
fn print_events_system(mut session: ResMut<P2PSession<GGRSConfig>>) {
    for event in session.events() {
        println!("GGRS Event: {:?}", event);
    }
}
*/

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    
    let mut app = App::new();


    let game_speed = GameSpeed::default();



    app.add_plugins(GgrsPlugin::<BoxConfig>::default())
    .set_rollback_schedule_fps(60)
    //.add_systems(ReadInputs, (input::input))
    .rollback_component_with_copy::<Player>()
            .rollback_component_with_copy::<LookingAt>()
        .rollback_component_with_copy::<Transform>()
        .rollback_component_with_copy::<Velocity>()
        .rollback_component_with_copy::<FrameCount>()
        .rollback_component_with_copy::<AmmunitionState>()
        .rollback_component_with_copy::<Projectile>()
        .rollback_component_with_copy::<Zombie>()
        .rollback_component_with_copy::<ZombieGame>()
        .rollback_component_with_copy::<ZombieSpawner>()
        .rollback_component_with_clone::<CharacterMovementState>()
        .rollback_component_with_clone::<BotDestination>()
        .rollback_component_with_copy::<Health>()
        .rollback_component_with_copy::<ProjectileCollider>()
        //.register_rollback_type::<HealthRegeneration>()
        .rollback_component_with_copy::<Death>()

        .add_systems(GgrsSchedule, (
            system_zombie_handle, apply_input_players, handle_weapon_input, system_interaction_player,
            update_velocity_player, system_move_zombie,

            move_players,
            movement_projectile,
            apply_velocity,

            system_health_player,
            system_zombie_game,

            increase_frame_system,
            system_end_game
        ))


    ;


    /*
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
        .build(&mut app);*/

    // Create an GGRS session
    app
    .insert_resource(LevelMapRequested{map: "maps/map_iso/iso_map.asset.ron".to_string(), level: "game/easy.level.ron".to_string()})
    .insert_resource(AvailableGameController{
        keyboard_mouse: true,
        gamepad: vec![]
    })
    .insert_resource(FrameCount { frame: 0 })
    .add_plugins(DefaultPlugins.set(WindowPlugin{
        primary_window: Some(Window{
        title: "Zombie".to_string(),
        //width: 500.,
        //height: 300.,
        resizable: true,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..Default::default()
        }),
        ..Default::default()
    }))
    .add_plugins((
        CharacterAnimationPlugin{ },
        AudioPlugin{},
        MapPlugin{},

        ZombieGamePlugin {},
        HomeMenuPlugin {},
    ));

    app.add_systems(Update, system_gamepad_event);

    app.add_systems(Startup, setup_player_camera);


    app.add_systems(OnEnter(GameState::PlayingZombie), (setup_zombie_game, setup_ingame_ui));

    app.add_systems(Update, (
        system_ingame_ui, system_weapon_ui, react_level_data, system_player_added
    ).run_if(in_state(GameState::PlayingZombie)));

    app.add_systems(OnExit(GameState::PlayingZombie), (
        system_unload_map,
        system_clear_ingame_ui,
        system_unload_players,
        system_unload_zombie_game,
    ));


    /*
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
*/

    //if opts.benchmark_mode {
    //  app.add_plugin(FPSPlugin{});
    //}

    app.add_plugins(WebPlugin{});

    app.run();
}

