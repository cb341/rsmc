use crate::prelude::*;

#[derive(Message)]
pub struct RemotePlayerSpawnedEvent {
    pub client_id: ClientId,
    pub position: Vec3,
}

#[derive(Message)]
pub struct RemotePlayerDespawnedEvent {
    pub client_id: ClientId,
}

#[derive(Message)]
pub struct RemotePlayerSyncEvent {
    pub players: HashMap<ClientId, PlayerState>,
}
