use crate::prelude::*;

pub fn broadcast_player_attributes_system(
    mut server: ResMut<RenetServer>,
    usernames: Res<ClientUsernames>,
    player_states: Res<player_resources::PlayerStates>,
) {
    for client_id in server.clients_id() {
        let mut other_player_states = player_states.players.clone();
        if let Some(username) = usernames.username_for_client_id(&client_id) {
            other_player_states.remove(username);
        }

        server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            bincode::serialize(&NetworkingMessage::PlayerSync(other_player_states)).unwrap(),
        );
    }
}
