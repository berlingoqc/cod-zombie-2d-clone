#![feature(derive_default_enum)]
#![feature(thread_local_internals)]
pub mod collider;
pub mod game;
pub mod map;
pub mod player;
pub mod zombies;
pub mod weapons;
pub mod utils;
pub mod health;
pub mod animation;
pub mod character;

use bevy::prelude::*;
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, MessageChannelMode, MessageChannelSettings, NetworkResource,
    ReliableChannelSettings,
};
use game::EntityId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const CLIENT_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 0,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 8,
    packet_buffer_size: 8,
};

pub const SERVER_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 1,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 8,
    packet_buffer_size: 8,
};

/*fn player_component_message_settings(channel: u8) -> MessageChannelSettings {
    MessageChannelSettings {
        channel,
        channel_mode: MessageChannelMode::Unreliable,
        message_buffer_size: 8,
        packet_buffer_size: 8,
    }
}*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Hello,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Welcome(EntityId)
}

pub fn setup_network_channels(mut net: ResMut<NetworkResource>) {
    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ClientMessage>(CLIENT_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<ServerMessage>(SERVER_MESSAGE_SETTINGS)
            .unwrap();
    });
}
