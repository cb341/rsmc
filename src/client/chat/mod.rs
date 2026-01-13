use crate::prelude::*;

pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ChatPlugin");

        app.add_systems(Startup, systems::setup_chat_container);
        app.add_systems(
            Update,
            (
                systems::handle_chat_message_sync_event,
                systems::add_message_to_chat_container_system,
                systems::chat_state_transition_system,
            ),
        );
        app.add_systems(
            Update,
            (
                systems::process_chat_input_system,
                systems::send_messages_system,
                systems::handle_chat_clear_events_system,
            )
                .run_if(in_state(GameState::Chatting)),
        );

        app.add_systems(OnEnter(GameState::Chatting), systems::focus_chat_system);
        app.add_systems(OnExit(GameState::Chatting), systems::unfocus_chat_system);

        app.insert_resource(resources::ChatHistory::default());
        app.insert_resource(resources::ChatState::default());

        app.add_message::<events::ChatSyncEvent>();
        app.add_message::<events::ChatMessageSendEvent>();
        app.add_message::<events::SingleChatSendEvent>();
        app.add_message::<events::ChatClearEvent>();
    }
}
