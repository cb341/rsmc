use crate::prelude::*;

pub mod events;
pub mod resources;
pub mod systems;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ChatPlugin");
        app.insert_resource(resources::ChatHistory::new());
        app.add_systems(
            Update,
            (
                chat_systems::sync_player_chat_messages_event,
                chat_systems::sync_single_player_chat_messages_system,
            ),
        );
        app.add_message::<chat_events::PlayerChatMessageSendEvent>();
        app.add_message::<chat_events::SyncPlayerChatMessagesEvent>();
    }
}
