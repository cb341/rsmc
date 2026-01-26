use std::time::UNIX_EPOCH;

use crate::prelude::*;

pub fn sync_single_player_chat_messages_system(
    mut server: ResMut<RenetServer>,
    mut player_send_messages: MessageReader<chat_events::PlayerChatMessageSendEvent>,
    mut chat_messages: ResMut<chat_resources::ChatHistory>,
) {
    for event in player_send_messages.read() {
        let message = event.message.clone();
        let sender = event.sender.clone();

        info!("Broadcasting message from sender {sender}");
        let message_count = chat_messages.messages.len();
        let message_id = message_count;

        let chat_message = ChatMessage {
            sender,
            message_id,
            message,
            timestamp: get_current_time_in_ms(),
        };

        chat_messages.messages.push(chat_message.clone());

        let response_message = NetworkingMessage::SingleChatMessageSync(chat_message);

        server.broadcast_message(
            DefaultChannel::ReliableOrdered,
            bincode::serialize(&response_message).unwrap(),
        );
    }
}

pub fn sync_player_chat_messages_event(
    mut server: ResMut<RenetServer>,
    mut events: MessageReader<chat_events::SyncPlayerChatMessagesEvent>,
    chat_messages: ResMut<chat_resources::ChatHistory>,
) {
    for event in events.read() {
        let client_id = event.client_id;
        info!("Synchronizing messages with client {}", client_id);

        let history = chat_messages.messages.clone();
        let response_message =
            bincode::serialize(&NetworkingMessage::ChatMessageSync(history)).unwrap();
        server.send_message(client_id, DefaultChannel::ReliableOrdered, response_message);
    }
}

fn get_current_time_in_ms() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH);
    match since_the_epoch {
        Ok(time) => match time.as_millis().try_into() {
            Ok(casted_time) => casted_time,
            Err(_error) => {
                error!("Could not cast time milis to u32");
                0
            }
        },
        Err(_error) => {
            error!("Could not fetch system time");
            0
        }
    }
}
