use crate::{networking::resources::ActiveConnections, prelude::*};

use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_states: ResMut<player_resources::PlayerStates>,
    mut past_block_updates: ResMut<terrain_resources::PastBlockUpdates>,
    mut chunk_manager: ResMut<ChunkManager>,
    client_usernames: Res<ClientUsernames>,
    mut request_queue: ResMut<terrain_resources::ClientChunkRequests>,
    mut accepted_clients: ResMut<ActiveConnections>,
    #[cfg(feature = "chat")] mut chat_message_events: MessageWriter<
        chat_events::PlayerChatMessageSendEvent,
    >,
) {
    for client_id in server.clients_id() {
        if !accepted_clients.is_accepted(&client_id) {
            continue;
        }

        let username = client_usernames
            .username_for_client_id(&client_id)
            .cloned()
            .unwrap_or(Username::from("unknown"));
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            let message = bincode::deserialize(&message).unwrap();

            match message {
                NetworkingMessage::BlockUpdate { position, block } => {
                    info!(
                        "Received block update from client {} {} {:?}",
                        client_id, position, block
                    );
                    chunk_manager.update_block(position, block);
                    past_block_updates
                        .updates
                        .push(terrain_events::BlockUpdateEvent { position, block });

                    server.broadcast_message_except(
                        client_id,
                        DefaultChannel::ReliableOrdered,
                        bincode::serialize(&NetworkingMessage::BlockUpdate { position, block })
                            .unwrap(),
                    );
                }
                #[cfg(feature = "chat")]
                NetworkingMessage::ChatMessageSend(message) => {
                    info!("Received chat message from {}", client_id);
                    chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                        sender: ChatMessageSender::Player(username.clone()),
                        message,
                    });
                }
                _ => {
                    warn!("Received unknown message type. (ReliabelOrdered)");
                }
            }
        }

        while let Some(message) =
            server.receive_message(client_id, DefaultChannel::ReliableUnordered)
        {
            let message = bincode::deserialize(&message).unwrap();
            debug!("Received message: {:?}", message);

            match message {
                NetworkingMessage::PlayerUpdate(player) => {
                    debug!(
                        "Received player update from client {} {}",
                        client_id, player.position
                    );
                    let username = client_usernames
                        .username_for_client_id(&client_id)
                        .expect("All clients should have associated username");
                    player_states.players.insert(username.clone(), player);
                }
                NetworkingMessage::ChunkBatchRequest(positions) => {
                    info!(
                        "Received chunk batch request at {:?} from client {}",
                        positions, client_id
                    );

                    request_queue.enqueue_bulk(client_id, &mut positions.into());
                }
                _ => {
                    warn!("Received unknown message type. (ReliableUnordered)");
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: MessageReader<ServerEvent>,
    mut player_states: ResMut<player_resources::PlayerStates>,
    past_block_updates: Res<terrain_resources::PastBlockUpdates>,
    mut request_queue: ResMut<terrain_resources::ClientChunkRequests>,
    mut client_usernames: ResMut<ClientUsernames>,
    mut active_connections: ResMut<ActiveConnections>,
    #[cfg(feature = "chat")] mut chat_message_events: MessageWriter<
        chat_events::PlayerChatMessageSendEvent,
    >,
    #[cfg(feature = "chat")] mut chat_sync_events: MessageWriter<
        chat_events::SyncPlayerChatMessagesEvent,
    >,
    transport: Res<NetcodeServerTransport>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let user_data = transport.user_data(*client_id).unwrap();
                let username = Username::from_user_data(&user_data);

                if let Some(existing_client_id) = client_usernames.get_client_id(&username) {
                    if active_connections.is_accepted(existing_client_id) {
                        server.send_message(
                            *client_id,
                            DefaultChannel::ReliableOrdered,
                            bincode::serialize(&NetworkingMessage::PlayerReject(String::from(
                                "Another Client is already connected with that Username.",
                            )))
                            .expect("Message should always be sendable"),
                        );
                        active_connections.reject(client_id);
                        server.disconnect(*client_id);
                        println!("Client {client_id} with Username '{username}' rejected");
                        break;
                    }
                }

                active_connections.accept(*client_id);

                player_states.players.insert(
                    username.clone(),
                    PlayerState {
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                    },
                );
                client_usernames.insert(*client_id, username.clone());
                server.send_message(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    bincode::serialize(&NetworkingMessage::PlayerAccept())
                        .expect("Message should always be sendable"),
                );
                println!("{username} connected");

                #[cfg(feature = "chat")]
                chat_sync_events.write(chat_events::SyncPlayerChatMessagesEvent {
                    client_id: *client_id,
                });

                #[cfg(feature = "chat")]
                chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                    sender: ChatMessageSender::Server,
                    message: format!("{username} joined the game"),
                });

                let message = bincode::serialize(&NetworkingMessage::PlayerJoin(username)).unwrap();
                server.broadcast_message_except(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    message,
                );

                for update in past_block_updates.updates.iter() {
                    let message = bincode::serialize(&NetworkingMessage::BlockUpdate {
                        position: update.position,
                        block: update.block,
                    })
                    .unwrap();
                    server.send_message(*client_id, DefaultChannel::ReliableOrdered, message);
                }
            }
            ServerEvent::ClientDisconnected { client_id, .. } => {
                request_queue.remove(client_id);
                if active_connections.is_accepted(client_id) {
                    active_connections.reject(client_id);

                    let username = client_usernames
                        .username_for_client_id(client_id)
                        .cloned()
                        .unwrap_or(Username("unknown".to_string()));

                    println!("Player {username} disconnected");

                    #[cfg(feature = "chat")]
                    chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                        sender: ChatMessageSender::Server,
                        message: format!("{username} left the game"),
                    });

                    if let Some(username) = client_usernames.username_for_client_id(client_id) {
                        let message =
                            bincode::serialize(&NetworkingMessage::PlayerLeave(username.clone()))
                                .unwrap();
                        server.broadcast_message(DefaultChannel::ReliableOrdered, message);
                    }
                }
            }
        }
    }
}

use bevy::ecs::message::MessageReader;
#[cfg(feature = "chat")]
use bevy::ecs::message::MessageWriter;
#[cfg(feature = "renet_visualizer")]
pub use server_visualizer::*;

#[cfg(feature = "renet_visualizer")]
pub mod server_visualizer {

    use crate::prelude::*;
    use bevy_inspector_egui::bevy_egui::EguiContexts;
    use renet_visualizer::RenetServerVisualizer;

    pub fn handle_events_for_visualizer_system(
        mut server_events: MessageReader<ServerEvent>,
        mut visualizer: ResMut<RenetServerVisualizer<200>>,
    ) {
        for event in server_events.read() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    visualizer.add_client(*client_id);
                }
                ServerEvent::ClientDisconnected { client_id, .. } => {
                    visualizer.remove_client(*client_id);
                }
            }
        }
    }

    pub fn update_visulizer_system(
        mut contexts: EguiContexts,
        mut visualizer: ResMut<RenetServerVisualizer<200>>,
        server: Res<RenetServer>,
    ) {
        visualizer.update(&server);

        let ctx = contexts
            .ctx_mut()
            .expect("egui is probably not loaded properly");

        egui::Window::new("Window").show(ctx, |ui| {
            ui.label("Windows can be moved by dragging them.");
            ui.label("They are automatically sized based on contents.");
            ui.label("You can turn on resizing and scrolling if you like.");
            ui.label("You would normally chose either panels OR windows.");
            visualizer.show_window(ctx);
        });
    }
}
