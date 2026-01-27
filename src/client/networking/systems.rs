use crate::prelude::*;

pub fn exit_on_last_window_closed_system(
    close_events: MessageReader<WindowCloseRequested>,
    windows: Query<(), With<Window>>,
    mut client: ResMut<RenetClient>,
    mut exit: MessageWriter<AppExit>,
) {
    if !close_events.is_empty() && windows.iter().count() <= 1 {
        client.disconnect();
        exit.write(AppExit::Success);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn receive_message_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut player_spawn_events: ResMut<Messages<remote_player_events::RemotePlayerSpawnedEvent>>,
    mut player_despawn_events: ResMut<Messages<remote_player_events::RemotePlayerDespawnedEvent>>,
    mut player_sync_events: ResMut<Messages<remote_player_events::RemotePlayerSyncEvent>>,
    mut block_update_events: ResMut<Messages<terrain_events::BlockUpdateEvent>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut chunk_mesh_events: ResMut<Messages<terrain_events::ChunkMeshUpdateEvent>>,
    mut world_regenerate_events: ResMut<Messages<terrain_events::WorldRegenerateEvent>>,
    #[cfg(feature = "chat")] mut chat_events: ResMut<Messages<chat_events::ChatSyncEvent>>,
    #[cfg(feature = "chat")] mut single_chat_events: ResMut<
        Messages<chat_events::SingleChatSendEvent>,
    >,
    mut exit_events: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        match bincode::deserialize(&message) {
            Ok(message) => match message {
                NetworkingMessage::PlayerReject(reject_reason) => {
                    eprintln!("Server connection rejected: {reject_reason}");
                    exit_events.write(AppExit::error());
                }
                NetworkingMessage::PlayerAccept(player_state) => {
                    commands.insert_resource(player_resources::LocalPlayerSpawnState(player_state));
                    commands.insert_resource(terrain_resources::SpawnArea::from_world_position(player_state.position.as_ivec3()));
                    next_state.set(GameState::LoadingSpawnArea);
                }
                NetworkingMessage::PlayerJoin(username) => {
                    player_spawn_events.write(remote_player_events::RemotePlayerSpawnedEvent {
                        username,
                        position: Vec3::ZERO,
                    });
                }
                NetworkingMessage::PlayerLeave(username) => {
                    player_despawn_events
                        .write(remote_player_events::RemotePlayerDespawnedEvent { username });
                }
                NetworkingMessage::BlockUpdate { position, block } => {
                    debug!("Client received block update message: {:?}", position);
                    block_update_events.write(terrain_events::BlockUpdateEvent {
                        position,
                        block,
                        from_network: true,
                    });
                }
                #[cfg(feature = "chat")]
                NetworkingMessage::ChatMessageSync(messages) => {
                    info!("Client received {} chat messages", messages.len());
                    chat_events.write(chat_events::ChatSyncEvent(messages));
                }
                #[cfg(feature = "chat")]
                NetworkingMessage::SingleChatMessageSync(message) => {
                    info!("Client received chat message {}", message.message);
                    single_chat_events.write(chat_events::SingleChatSendEvent(message));
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
                        chunk_mesh_events
                            .write(terrain_events::ChunkMeshUpdateEvent { chunk_position });
                    }
                }
                NetworkingMessage::PlayerSync(event) => {
                    player_sync_events
                        .write(remote_player_events::RemotePlayerSyncEvent { players: event });
                }
                NetworkingMessage::ServerAsksClientNicelyToRerequestChunkBatch() => {
                    info!("Client asked for chunk batch.");
                    world_regenerate_events.write(terrain_events::WorldRegenerateEvent);
                }
                _ => {
                    warn!("Received unknown message type. (ReliableUnordered)");
                }
            }
        }
    }
}
