use crate::{
    networking::resources::{ActiveConnections, PendingDisconnects},
    prelude::*,
};

use bevy::prelude::*;

fn find_ground_spawn_position(chunk_manager: &ChunkManager, base_world_position: IVec3) -> IVec3 {
    const MAX_DELTA: i32 = 64;

    for dy in 0..MAX_DELTA {
        let pos_down = base_world_position + IVec3::new(0, -dy, 0);
        if is_standable(chunk_manager, pos_down) {
            return pos_down;
        }

        let pos_up = base_world_position + IVec3::new(0, dy, 0);
        if is_standable(chunk_manager, pos_up) {
            return pos_up;
        }
    }

    warn!("No standable ground found near {base_world_position}, using as-is");
    base_world_position
}

fn is_standable(chunk_manager: &ChunkManager, world_position: IVec3) -> bool {
    let legs_block = chunk_manager.get_block(world_position);
    let head_block = chunk_manager.get_block(world_position + IVec3::Y);
    let ground_block = chunk_manager.get_block(world_position - IVec3::Y);

    legs_block.is_some_and(|b| b.is_walkable())
        && head_block.is_some_and(|b| b.is_walkable())
        && ground_block.is_some_and(|b| b.is_standable())
}

pub fn disconnect_all_clients_on_exit_system(
    mut server: ResMut<RenetServer>,
    mut exit_events: MessageReader<AppExit>,
) {
    if exit_events.read().len() > 0 {
        server.disconnect_all();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_states: ResMut<player_resources::PlayerStates>,
    mut past_block_updates: ResMut<terrain_resources::PastBlockUpdates>,
    mut chunk_manager: ResMut<ChunkManager>,
    client_usernames: Res<ClientUsernames>,
    mut request_queue: ResMut<terrain_resources::ClientChunkRequests>,
    accepted_clients: Res<ActiveConnections>,
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
            .expect("All clients should be associated with a username");
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
                        sender: ChatMessageSender::Player(username),
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
                    player_states.players.insert(*username, player);
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
    mut pending_disconnects: ResMut<PendingDisconnects>,
    #[cfg(feature = "chat")] mut chat_message_events: MessageWriter<
        chat_events::PlayerChatMessageSendEvent,
    >,
    #[cfg(feature = "chat")] mut chat_sync_events: MessageWriter<
        chat_events::SyncPlayerChatMessagesEvent,
    >,
    transport: Res<NetcodeServerTransport>,
    chunk_manager: Res<ChunkManager>,
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
                                "Another Client is already connected with that Username. Wait 15 seconds before trying again.",
                            )))
                            .expect("Message should always be sendable"),
                        );
                        active_connections.reject(client_id);
                        pending_disconnects.queue(*client_id);
                        println!("Client {client_id} with Username '{username}' rejected");
                        continue;
                    }
                }

                active_connections.accept(*client_id);

                let player_state = player_states.players.entry(username).or_insert_with(|| {
                    let ground_pos =
                        find_ground_spawn_position(&chunk_manager, DEFAULT_SPAWN_POINT.as_ivec3());
                    PlayerState {
                        position: ground_pos.as_vec3(),
                        rotation: Quat::IDENTITY,
                    }
                });

                client_usernames.insert(*client_id, username);
                server.send_message(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    bincode::serialize(&NetworkingMessage::PlayerAccept(*player_state))
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
                        .expect("All clients should have an associated username");

                    println!("Player {username} disconnected");

                    #[cfg(feature = "chat")]
                    chat_message_events.write(chat_events::PlayerChatMessageSendEvent {
                        sender: ChatMessageSender::Server,
                        message: format!("{username} left the game"),
                    });

                    if let Some(username) = client_usernames.username_for_client_id(client_id) {
                        let message =
                            bincode::serialize(&NetworkingMessage::PlayerLeave(*username))
                                .expect("Messages should be serializable");
                        server.broadcast_message(DefaultChannel::ReliableOrdered, message);
                    }
                }
            }
        }
    }
}

pub fn process_pending_disconnects_system(
    mut server: ResMut<RenetServer>,
    mut pending_disconnects: ResMut<PendingDisconnects>,
) {
    for client_id in pending_disconnects.drain_ready() {
        server.disconnect(client_id);
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
