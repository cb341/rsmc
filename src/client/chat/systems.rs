use crate::prelude::*;
use bevy::input::{keyboard::KeyboardInput, ButtonState};
use bevy_flair::style::components::{ClassList, NodeStyleSheet};
use chat_events::ChatMessageSendEvent;

const MESSAGE_PROMPT: &str = "> ";
const MAX_MESSAGE_LENGTH: usize = 42;

pub fn setup_chat_container(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node::default(),
            Name::new("root"),
            NodeStyleSheet::new(asset_server.load("chat.css")),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node::default(),
                Name::new("chat_message_container"),
                ClassList::empty(),
                chat_components::ChatMessageContainer,
                Text::new(""),
            ));

            parent.spawn((
                Node::default(),
                Name::new("chat_message_input"),
                ClassList::empty(),
                chat_components::ChatMessageInputElement,
                Text::new(MESSAGE_PROMPT),
            ));
        });
}

pub fn send_messages_system(
    mut client: ResMut<RenetClient>,
    mut event_reader: MessageReader<ChatMessageSendEvent>,
) {
    for event in event_reader.read() {
        let message = event.0.clone();

        info!("Sending message \"{}\" to server", message);

        client.send_message(
            DefaultChannel::ReliableOrdered,
            bincode::serialize(&NetworkingMessage::ChatMessageSend(message)).unwrap(),
        );
    }
}

pub fn chat_state_transition_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut chat_state: ResMut<chat_resources::ChatState>,
) {
    let current_state_value = current_state.get();
    let mut next_state_value = current_state_value.clone();

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        info!("Focusing chat via KeyT");
        if *current_state_value == GameState::Playing {
            chat_state.just_focused = true;
            next_state_value = GameState::Chatting;
        }
    }
    if keyboard_input.just_pressed(KeyCode::Escape) && *current_state_value == GameState::Chatting {
        info!("Unfocusing chat via Escape");
        next_state_value = GameState::Playing;
    }

    next_state.set(next_state_value);
}

pub fn process_chat_input_system(
    mut evr_kbd: MessageReader<KeyboardInput>,
    mut chat_input_query: Query<&mut Text, With<chat_components::ChatMessageInputElement>>,
    mut send_event_writer: MessageWriter<ChatMessageSendEvent>,
    mut chat_state: ResMut<chat_resources::ChatState>,
    mut chat_clear_writer: MessageWriter<chat_events::ChatClearEvent>,
) {
    let mut text = chat_input_query
        .single_mut()
        .expect("chat_input doesn't exist");
    let mut chat_input_value = text.0.clone();

    for event in evr_kbd.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if chat_state.just_focused {
            // Hack to prevent 'T' from being added to the chat input upon focus
            chat_state.just_focused = false;
            continue;
        }

        info!("Chat state: {}", chat_input_value);

        let mut message = extract_message(&chat_input_value);

        match &event.logical_key {
            Key::Enter if !message.trim().is_empty() => {
                if message.trim() == "CLEAR" {
                    chat_clear_writer.write(chat_events::ChatClearEvent);
                } else {
                    send_event_writer.write(ChatMessageSendEvent(message.trim().to_string()));
                }
                message.clear();
            }
            Key::Backspace => {
                message.pop();
            }
            Key::Space => {
                if message.len() < MAX_MESSAGE_LENGTH {
                    message.push(' ');
                }
            }
            Key::Character(input) => {
                if message.len() < MAX_MESSAGE_LENGTH && input.chars().all(|c| !c.is_control()) {
                    message.push_str(input);
                }
            }
            _ => {}
        }

        chat_input_value = MESSAGE_PROMPT.to_string() + &message;
    }

    text.clear();

    text.0 += &chat_input_value;
}

fn extract_message(value: &str) -> String {
    let message = value.trim_start_matches(MESSAGE_PROMPT);
    message.to_string()
}

pub fn handle_chat_message_sync_event(
    mut sync_events: MessageReader<chat_events::ChatSyncEvent>,
    mut send_events: MessageWriter<chat_events::SingleChatSendEvent>,
) {
    for event in sync_events.read() {
        event.0.clone().into_iter().for_each(|message| {
            send_events.write(chat_events::SingleChatSendEvent(message));
        })
    }
}

pub fn add_message_to_chat_container_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &chat_components::ChatMessageContainer,
        &mut ScrollPosition,
    )>,
    mut events: MessageReader<chat_events::SingleChatSendEvent>,
) {
    for event in events.read() {
        let (entity, _, mut scroll_position) = single_mut!(query);

        // Offset does not need to be exact, just needs to be large enough to see the new message
        scroll_position.0 += 100.0;

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Node::default(),
                Name::new("chat_entry"),
                chat_components::ChatMessageElement,
                Text::new(event.0.message.clone()),
            ));
        });
    }
}

pub fn handle_chat_clear_events_system(
    mut chat_clear_events: MessageReader<chat_events::ChatClearEvent>,
    mut commands: Commands,
    query: Query<Entity, With<chat_components::ChatMessageContainer>>,
) {
    for _ in chat_clear_events.read() {
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn_related::<Children>();
        }
    }
}

pub fn unfocus_chat_system(
    mut chat_container_query: Query<
        &mut ClassList,
        (
            With<chat_components::ChatMessageContainer>,
            Without<chat_components::ChatMessageInputElement>,
        ),
    >,
    mut chat_input_query: Query<
        &mut ClassList,
        (
            With<chat_components::ChatMessageInputElement>,
            Without<chat_components::ChatMessageContainer>,
        ),
    >,
) {
    let mut container_classes = single_mut!(chat_container_query);
    let mut input_classes = single_mut!(chat_input_query);

    info!("Handling unfocus state");
    container_classes.remove("focused");
    container_classes.add("unfocused");

    input_classes.remove("focused");
    input_classes.add("unfocused");
}

pub fn focus_chat_system(
    mut chat_container_query: Query<
        &mut ClassList,
        (
            With<chat_components::ChatMessageContainer>,
            Without<chat_components::ChatMessageInputElement>,
        ),
    >,
    mut chat_input_query: Query<
        &mut ClassList,
        (
            With<chat_components::ChatMessageInputElement>,
            Without<chat_components::ChatMessageContainer>,
        ),
    >,
) {
    let mut container_classes = single_mut!(chat_container_query);
    let mut input_classes = single_mut!(chat_input_query);

    info!("Handling focus state");
    container_classes.add("focused");
    container_classes.remove("unfocused");

    input_classes.add("focused");
    input_classes.remove("unfocused");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatMessage;
    use chat_events::{ChatClearEvent, SingleChatSendEvent};

    fn get_chat_messages(app: &mut App) -> Vec<String> {
        let mut messages = app
            .world_mut()
            .query::<(&Text, &chat_components::ChatMessageElement)>();

        messages
            .iter(app.world())
            .map(|(text, _)| text.0.clone())
            .collect()
    }

    #[test]
    fn test_send_message_system() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_systems(Update, add_message_to_chat_container_system)
            .insert_resource(Messages::<SingleChatSendEvent>::default());

        app.world_mut().spawn((
            ScrollPosition::default(),
            chat_components::ChatMessageContainer,
        ));

        let mut event_writer = app
            .world_mut()
            .get_resource_mut::<Messages<SingleChatSendEvent>>()
            .unwrap();

        event_writer.write(SingleChatSendEvent(ChatMessage {
            message: "Hello World".to_string(),
            client_id: 0,
            message_id: 1,
            timestamp: 0,
        }));

        app.update();

        let mut messages = app
            .world_mut()
            .query::<(&Text, &chat_components::ChatMessageElement)>();

        let message_count = messages.iter(app.world()).count();
        assert_eq!(message_count, 1);
        assert_eq!(
            messages.iter(app.world()).next().unwrap().0 .0,
            "Hello World"
        );
    }

    #[test]
    fn test_chat_clear_system() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_systems(Update, handle_chat_clear_events_system)
            .insert_resource(Messages::<ChatClearEvent>::default());

        app.world_mut()
            .spawn(chat_components::ChatMessageContainer)
            .with_children(|parent| {
                parent.spawn((
                    Node::default(),
                    chat_components::ChatMessageElement,
                    Text::new("Message 1"),
                ));

                parent.spawn((
                    Node::default(),
                    chat_components::ChatMessageElement,
                    Text::new("Message 2"),
                ));
            });

        let messages = get_chat_messages(&mut app);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "Message 1");
        assert_eq!(messages[1], "Message 2");

        let mut event_writer = app
            .world_mut()
            .get_resource_mut::<Messages<chat_events::ChatClearEvent>>()
            .unwrap();
        event_writer.write(chat_events::ChatClearEvent);

        app.update();

        let messages = get_chat_messages(&mut app);
        assert_eq!(messages.len(), 0);
    }
}
