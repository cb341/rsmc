use crate::prelude::*;

pub fn handle_block_update_events(
    mut chunk_manager: ResMut<ChunkManager>,
    mut block_update_events: EventReader<terrain_events::BlockUpdateEvent>,
    mut chunk_mesh_update_events: EventWriter<terrain_events::ChunkMeshUpdateEvent>,
    mut player_collider_events: EventWriter<player_events::PlayerColliderUpdateEvent>,
    mut client: ResMut<RenetClient>,
) {
    for event in block_update_events.read() {
        info!("Block update message: {:?}", event.position);
        chunk_manager
            .update_block(event.position, event.block)
            .iter()
            .for_each(|affected_chunk_position| {
                chunk_mesh_update_events.send(terrain_events::ChunkMeshUpdateEvent {
                    chunk_position: *affected_chunk_position,
                });
            });

        player_collider_events.send(player_events::PlayerColliderUpdateEvent);

        if !event.from_network {
            info!("sending block update event");
            client.send_message(
                DefaultChannel::ReliableOrdered,
                bincode::serialize(&NetworkingMessage::BlockUpdate {
                    position: event.position,
                    block: event.block,
                })
                .unwrap(),
            );
        }
    }
}

pub fn handle_player_collider_events_system(
    mut player_collider_events: EventReader<player_events::PlayerColliderUpdateEvent>,
    mut query: Query<(&mut Transform, &player_components::Player)>,
    mut collider_events: EventWriter<collider_events::ColliderUpdateEvent>,
) {
    if player_collider_events.read().count() == 0 {
        return;
    }

    for (transform, _) in query.iter_mut() {
        let player_position = transform.translation.floor();

        collider_events.send(collider_events::ColliderUpdateEvent {
            grid_center_position: player_position.into(),
        });
    }
}
