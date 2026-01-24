use std::time::Duration;

use bevy::math::{IVec3, Quat, Vec3};
use bevy_renet::netcode::NETCODE_USER_DATA_BYTES;
use chrono::DateTime;
use renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{BlockId, Chunk};

pub const SERVER_MESSAGE_ID: ClientId = 0;

pub struct Username(pub String);
impl Username {
    pub fn to_netcode_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        if self.0.len() > NETCODE_USER_DATA_BYTES - 8 {
            panic!("Username is too big");
        }
        user_data[0] = self.0.len() as u8;
        user_data[1..self.0.len() + 1].copy_from_slice(self.0.as_bytes());

        user_data
    }

    pub fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        let mut len = user_data[0] as usize;
        len = len.min(NETCODE_USER_DATA_BYTES - 1);
        let data = user_data[1..len + 1].to_vec();
        let username = String::from_utf8(data).unwrap_or("unknown".to_string());
        Self(username)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub client_id: ClientId,
    pub message_id: usize,
    pub timestamp: i64,
    pub message: String,
}

impl ChatMessage {
    pub fn format_string(&self) -> String {
        let dt = DateTime::from_timestamp_millis(self.timestamp).expect("invalid timestamp");
        let timestamp_string = dt.to_string();

        let client_name = match self.client_id {
            SERVER_MESSAGE_ID => "SERVER".to_string(),
            _ => self.client_id.to_string(),
        };

        format!("[{}] {}: {}", timestamp_string, client_name, self.message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkingMessage {
    PlayerJoin(ClientId, String),
    PlayerLeave(ClientId),
    PlayerUpdate(PlayerState),
    PlayerSync(HashMap<ClientId, PlayerState>),
    ChunkBatchRequest(Vec<IVec3>),
    ChunkBatchResponse(Vec<Chunk>),
    ChatMessageSend(String),
    SingleChatMessageSync(ChatMessage),
    ChatMessageSync(Vec<ChatMessage>),
    BlockUpdate { position: IVec3, block: BlockId },
    ServerAsksClientNicelyToRerequestChunkBatch(),
}

const CHANNELS: [ChannelConfig; 3] = [
    ChannelConfig {
        channel_id: 0,
        max_memory_usage_bytes: 1024 * 1024 * 1024 * 1024,
        send_type: SendType::Unreliable,
    },
    ChannelConfig {
        channel_id: 1,
        max_memory_usage_bytes: 1024 * 1024 * 1024 * 1024,
        send_type: SendType::ReliableOrdered {
            resend_time: Duration::from_millis(300),
        },
    },
    ChannelConfig {
        channel_id: 2,
        max_memory_usage_bytes: 1024 * 1024 * 1024 * 1024,
        send_type: SendType::ReliableUnordered {
            resend_time: Duration::from_millis(300),
        },
    },
];

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        client_channels_config: CHANNELS.to_vec(),
        ..Default::default()
    }
}
