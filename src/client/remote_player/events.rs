use crate::prelude::*;

#[derive(Message)]
pub struct RemotePlayerSpawnedEvent {
    pub username: Username,
    pub position: Vec3,
}

#[derive(Message)]
pub struct RemotePlayerDespawnedEvent {
    pub username: Username,
}

#[derive(Message)]
pub struct RemotePlayerSyncEvent {
    pub players: HashMap<Username, PlayerState>,
}
