mod game;
mod server;

use bevy::{
    app::ScheduleRunnerSettings,
    prelude::App,
    prelude::{Component, EventReader, ResMut},
    MinimalPlugins, asset::AssetPlugin,
};
use bevy::{math::vec2, prelude::*};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, net::Ipv4Addr, net::SocketAddr, time::Duration};
use turbulence::message_channels::ChannelMessage;


use server::*;
use game::*;
use shared::game::ZombieGamePlugin;

#[derive(Component, Serialize, Deserialize)]
struct NetworkHandle(u32);

fn main() {
    App::new()
        // Networking setup
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            1000 / 30,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(shared::setup_network_channels)
        .add_startup_system(setup_server)
        .add_system(network_events)

        // Game setup
        
        .add_plugin(ZombieGamePlugin{})
        .add_startup_system(setup_zombie_game)
        //.add_system(arugio_shared::update_velocity)
        //.add_system(arugio_shared::update_position)
        //.add_system(spawn_ball_system)
        //.add_system(unowned_ball_input)
        //.add_system_to_stage(CoreStage::PreUpdate, read_component_channel::<Position>)
        //.add_system_to_stage(
        //    CoreStage::PreUpdate,
        //    read_component_channel::<TargetVelocity>,
        //)

        // 
        .add_system_to_stage(CoreStage::PreUpdate, read_network_channels)
        .add_system_to_stage(CoreStage::PostUpdate, broadcast_changes)
        .run();
}



fn spawn_ball_system(
    mut cmd: Commands,
//    unowned_balls: Query<&BallId, Without<NetworkHandle>>
) {
    /*let mut count = 0;
    let mut highest_id = 0;
    for ball in unowned_balls.iter() {
        count += 1;
        highest_id = highest_id.max(ball.0);
    }

    if count < 3 {
        cmd.spawn_bundle((
            BallId(highest_id + 1),
            Position(vec2(
                rand::random::<f32>() * 10.0 - 5.0,
                rand::random::<f32>() * 10.0 - 5.0,
            )),
            Velocity::default(),
            TargetVelocity::default(),
        ));

        println!("Spawned ball {:?}", highest_id + 1);
    }
    */
}

fn unowned_ball_input(
    //mut unowned_balls: Query<(&BallId, &mut TargetVelocity), Without<NetworkHandle>>,
) {
    /*
    for (_, mut target_velocity) in unowned_balls.iter_mut() {
        target_velocity.0.x = rand::random::<f32>() * 2.0 - 1.0;
        target_velocity.0.y = rand::random::<f32>() * 2.0 - 1.0;
    }
    */
}

