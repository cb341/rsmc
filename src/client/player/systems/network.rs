use crate::prelude::*;

pub fn broadcast_player_attributes_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&player_components::Player, &Transform)>,
    camera_query: Query<(&Camera3d, &player_components::PlayerCamera, &Transform)>,
) {
    if query.is_empty() {
        return;
    }

    let (_, transform) = single!(query);
    let (_, _, camera_transform) = single!(camera_query);

    let player_state = PlayerState {
        position: transform.translation,
        rotation: camera_transform.rotation,
    };

    client.send_message(
        DefaultChannel::ReliableUnordered,
        bincode::serialize(&NetworkingMessage::PlayerUpdate(player_state)).unwrap(),
    );
}
