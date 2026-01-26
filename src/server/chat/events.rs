use crate::prelude::*;

#[derive(Message)]
pub struct PlayerChatMessageSendEvent {
    pub sender: ChatMessageSender,
    pub message: String,
}

#[derive(Message)]
pub struct SyncPlayerChatMessagesEvent {
    pub client_id: ClientId,
}
