use crate::prelude::*;

#[derive(Message)]
pub struct PlayerChatMessageSendEvent {
    pub client_id: ClientId,
    pub message: String,
}

#[derive(Message)]
pub struct SyncPlayerChatMessagesEvent {
    pub client_id: ClientId,
}
