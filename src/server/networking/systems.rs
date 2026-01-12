use crate::prelude::*;

pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_states: ResMut<player_resources::PlayerStates>,
    mut past_block_updates: ResMut<terrain_resources::PastBlockUpdates>,
    chunk_manager: ResMut<ChunkManager>,
    #[cfg(feature = "chat")] mut chat_message_events: EventWriter<
        chat_events::PlayerChatMessageSendEvent,
    >,
    generator: Res<terrain_resources::Generator>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            let message = bincode::deserialize(&message).unwrap();

            match message {
                NetworkingMessage::BlockUpdate { position, block } => {
                    info!(
                        "Received block update from client {} {} {:?}",
                        client_id, position, block
                    );
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
                    chat_message_events
                        .write(chat_events::PlayerChatMessageSendEvent { client_id, message });
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
                    player_states.players.insert(client_id, player);
                }
                NetworkingMessage::ChunkBatchRequest(positions) => {
                    info!(
                        "Received chunk batch request at {:?} from client {}",
                        positions, client_id
                    );

                    let chunks: Vec<Chunk> = positions
                        .into_par_iter()
                        .map(|position| {
                            let chunk = chunk_manager.get_chunk(position);

                            match chunk {
                                Some(chunk) => *chunk,
                                None => {
                                    let mut chunk = Chunk::new(position);
                                    generator.generate_chunk(&mut chunk);
                                    chunk
                                }
                            }
                        })
                        .collect();

                    let message =
                        bincode::serialize(&NetworkingMessage::ChunkBatchResponse(chunks));

                    server.send_message(
                        client_id,
                        DefaultChannel::ReliableUnordered,
                        message.unwrap(),
                    );
                }
                _ => {
                    warn!("Received unknown message type. (ReliableUnordered)");
                }
            }
        }
    }
}

pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_states: ResMut<player_resources::PlayerStates>,
    past_block_updates: Res<terrain_resources::PastBlockUpdates>,
    #[cfg(feature = "chat")] mut chat_message_events: EventWriter<
        chat_events::PlayerChatMessageSendEvent,
    >,
    #[cfg(feature = "chat")] mut chat_sync_events: EventWriter<
        chat_events::SyncPlayerChatMessagesEvent,
    >,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {client_id} connected");
                player_states.players.insert(
                    *client_id,
                    PlayerState {
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                    },
                );

                #[cfg(feature = "chat")]
                chat_sync_events.write(chat_events::SyncPlayerChatMessagesEvent {
                    client_id: *client_id,
                });

                #[cfg(feature = "chat")]
                chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                    client_id: SERVER_MESSAGE_ID,
                    message: format!("Player {} joined the game", *client_id),
                });

                let message =
                    bincode::serialize(&NetworkingMessage::PlayerJoin(*client_id)).unwrap();
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
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {client_id} disconnected: {reason}");
                player_states.players.remove(client_id);

                #[cfg(feature = "chat")]
                chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                    client_id: SERVER_MESSAGE_ID,
                    message: format!("Player {} left the game", client_id),
                });

                let message =
                    bincode::serialize(&NetworkingMessage::PlayerLeave(*client_id)).unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
        }
    }
}

#[cfg(feature = "renet_visualizer")]
pub use server_visualizer::*;

#[cfg(feature = "renet_visualizer")]
pub mod server_visualizer {

    use crate::prelude::*;
    use bevy_inspector_egui::bevy_egui::EguiContexts;
    use renet_visualizer::RenetServerVisualizer;

    pub fn handle_events_for_visualizer_system(
        mut server_events: EventReader<ServerEvent>,
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
        mut egui_contexts: EguiContexts,
        mut visualizer: ResMut<RenetServerVisualizer<200>>,
        server: Res<RenetServer>,
    ) {
        visualizer.update(&server);
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}
