use std::{fmt::Display, time::Duration};

use bevy::{
    ecs::resource::Resource,
    math::{IVec3, Quat, Vec3},
};
use bevy_renet::netcode::NETCODE_USER_DATA_BYTES;
use chrono::DateTime;
use renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{BlockId, Chunk};

pub const SERVER_USERNAME: &str = "SERVER";
pub const MAX_USERNAME_LENGTH_BYTES: usize = 50;

#[derive(Resource, Default)]
pub struct ClientUsernames {
    client_to_username: HashMap<ClientId, Username>,
    username_to_client: HashMap<Username, ClientId>,
}

impl ClientUsernames {
    pub fn insert(&mut self, client_id: ClientId, username: Username) {
        self.client_to_username.insert(client_id, username.clone());
        self.username_to_client.insert(username, client_id);
    }

    pub fn get_client_id(&self, username: &Username) -> Option<&ClientId> {
        self.username_to_client.get(username)
    }

    pub fn username_for_client_id(&self, client_id: &ClientId) -> Option<&Username> {
        self.client_to_username.get(client_id)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone)]
pub struct Username(pub String);

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Username {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

impl From<String> for Username {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Username {
    pub fn is_server(&self) -> bool {
        self.0.eq(SERVER_USERNAME)
    }
    pub fn to_netcode_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        if self.0.len() > MAX_USERNAME_LENGTH_BYTES {
            panic!(
                "Username is too big: has {} bytes out of max {MAX_USERNAME_LENGTH_BYTES}",
                self.0.len()
            );
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
pub enum ChatMessageSender {
    Player(Username),
    Server,
}

impl Display for ChatMessageSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: &str = match self {
            ChatMessageSender::Player(username) => &username.0,
            ChatMessageSender::Server => "SERVER",
        };
        write!(f, "{}", str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub sender: ChatMessageSender,
    pub message_id: usize,
    pub timestamp: i64,
    pub message: String,
}

impl ChatMessage {
    pub fn format_string(&self) -> String {
        let dt = DateTime::from_timestamp_millis(self.timestamp).expect("invalid timestamp");
        let timestamp_string = dt.to_string();

        let username = &self.sender;
        let message = &self.message;

        format!("[{timestamp_string}] {username}: {message}")
    }
}

type RejectReason = String;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkingMessage {
    PlayerAccept(),
    PlayerReject(RejectReason),
    PlayerJoin(Username),
    PlayerLeave(Username),
    PlayerUpdate(PlayerState),
    PlayerSync(HashMap<Username, PlayerState>),
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
