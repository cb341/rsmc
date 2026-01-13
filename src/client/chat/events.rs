use crate::prelude::*;

#[derive(Message)]
pub struct ChatSyncEvent(pub Vec<ChatMessage>);

#[derive(Message)]
pub struct SingleChatSendEvent(pub ChatMessage);

#[derive(Message)]
pub struct ChatMessageSendEvent(pub String);

#[derive(Message)]
pub struct ChatClearEvent;
