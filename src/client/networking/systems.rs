use crate::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut player_spawn_events: ResMut<Events<remote_player_events::RemotePlayerSpawnedEvent>>,
    mut player_despawn_events: ResMut<Events<remote_player_events::RemotePlayerDespawnedEvent>>,
    mut player_sync_events: ResMut<Events<remote_player_events::RemotePlayerSyncEvent>>,
    mut block_update_events: ResMut<Events<terrain_events::BlockUpdateEvent>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut chunk_mesh_events: ResMut<Events<terrain_events::ChunkMeshUpdateEvent>>,
    mut world_regenerate_events: ResMut<Events<terrain_events::WorldRegenerateEvent>>,
    #[cfg(feature = "chat")] mut chat_events: ResMut<Events<chat_events::ChatSyncEvent>>,
    #[cfg(feature = "chat")] mut single_chat_events: ResMut<
        Events<chat_events::SingleChatSendEvent>,
    >,
    mut spawn_area_loaded: ResMut<terrain_resources::SpawnAreaLoaded>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message) {
            Ok(message) => match message {
                NetworkingMessage::PlayerJoin(event) => {
                    player_spawn_events.send(remote_player_events::RemotePlayerSpawnedEvent {
                        client_id: event,
                        position: Vec3::ZERO,
                    });
                }
                NetworkingMessage::PlayerLeave(event) => {
                    player_despawn_events.send(remote_player_events::RemotePlayerDespawnedEvent {
                        client_id: event,
                    });
                }
                NetworkingMessage::BlockUpdate { position, block } => {
                    debug!("Client received block update message: {:?}", position);
                    block_update_events.send(terrain_events::BlockUpdateEvent {
                        position,
                        block,
                        from_network: true,
                    });
                }
                #[cfg(feature = "chat")]
                NetworkingMessage::ChatMessageSync(messages) => {
                    info!("Client received {} chat messages", messages.len());
                    chat_events.send(chat_events::ChatSyncEvent(messages));
                }
                #[cfg(feature = "chat")]
                NetworkingMessage::SingleChatMessageSync(message) => {
                    info!("Client received chat message {}", message.message);
                    single_chat_events.send(chat_events::SingleChatSendEvent(message));
                }
                _ => {
                    warn!("Received unknown message type. (ReliableOrdered)");
                }
            },
            Err(message) => {
                error!("Could not deserialize message {:?}", message);
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::ReliableUnordered) {
        let message = bincode::deserialize(&message);

        if message.is_err() {
            error!("Failed to deserialize message.");
            continue;
        }

        if let Ok(message) = message {
            debug!("Received message: {:?}", message);
            match message {
                NetworkingMessage::ChunkBatchResponse(chunks) => {
                    info!("Client received chunk batch response message.");
                    for chunk in chunks {
                        info!(
                            "Client received chunk response message for: {:?}",
                            chunk.position
                        );
                        let chunk_position = chunk.position;
                        chunk_manager.insert_chunk(chunk);
                        chunk_mesh_events.send(terrain_events::ChunkMeshUpdateEvent {
                            chunk_position,
                        });

                        if chunk_position.eq(&IVec3::ZERO) {
                            info!("Spawn area loaded.");
                            spawn_area_loaded.0 = true;
                        }
                    }
                }
                NetworkingMessage::PlayerSync(event) => {
                    player_sync_events
                        .send(remote_player_events::RemotePlayerSyncEvent { players: event });
                }
                NetworkingMessage::ServerAsksClientNicelyToRerequestChunkBatch() => {
                    info!("Client asked for chunk batch.");
                    world_regenerate_events.send(terrain_events::WorldRegenerateEvent);
                }
                _ => {
                    warn!("Received unknown message type. (ReliableUnordered)");
                }
            }
        }
    }
}
