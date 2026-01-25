use std::{
    fmt::{Debug, Display},
    time::Duration,
};

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
        self.client_to_username.insert(client_id, username);
        self.username_to_client.insert(username, client_id);
    }

    pub fn get_client_id(&self, username: &Username) -> Option<&ClientId> {
        self.username_to_client.get(username)
    }

    pub fn username_for_client_id(&self, client_id: &ClientId) -> Option<&Username> {
        self.client_to_username.get(client_id)
    }
}

const USERNAME_BUFFER_SIZE: usize = MAX_USERNAME_LENGTH_BYTES + 1;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Username([u8; USERNAME_BUFFER_SIZE]);

impl Username {
    pub fn new(s: &str) -> Result<Self, String> {
        if s.len() > MAX_USERNAME_LENGTH_BYTES {
            return Err(format!(
                "Username too long: {} bytes (max {MAX_USERNAME_LENGTH_BYTES})",
                s.len()
            ));
        }
        let mut buf = [0u8; USERNAME_BUFFER_SIZE];
        buf[0] = s.len() as u8;
        buf[1..=s.len()].copy_from_slice(s.as_bytes());
        Ok(Self(buf))
    }

    pub fn as_str(&self) -> &str {
        let len = self.0[0] as usize;
        std::str::from_utf8(&self.0[1..=len]).unwrap_or("invalid")
    }

    pub fn is_server(&self) -> bool {
        self.as_str() == SERVER_USERNAME
    }

    pub fn to_netcode_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        let len = self.0[0] as usize;
        user_data[..=len].copy_from_slice(&self.0[..=len]);
        user_data
    }

    pub fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        let mut len = user_data[0] as usize;
        len = len.min(MAX_USERNAME_LENGTH_BYTES);
        let mut buf = [0u8; USERNAME_BUFFER_SIZE];
        buf[..=len].copy_from_slice(&user_data[..=len]);
        Self(buf)
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Username({:?})", self.as_str())
    }
}

impl From<&str> for Username {
    fn from(value: &str) -> Self {
        Self::new(value).unwrap()
    }
}

impl From<String> for Username {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Serialize for Username {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

pub const DEFAULT_SPAWN_POINT: Vec3 = Vec3::new(0.0, 43.0, 0.0); // TODO: determine spawn point from terain

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PlayerState {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: DEFAULT_SPAWN_POINT,
            rotation: Quat::IDENTITY,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChatMessageSender {
    Player(Username),
    Server,
}

impl Display for ChatMessageSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatMessageSender::Player(username) => write!(f, "{username}"),
            ChatMessageSender::Server => write!(f, "SERVER"),
        }
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
    PlayerAccept(PlayerState),
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
