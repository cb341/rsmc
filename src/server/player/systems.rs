use crate::networking::resources::ActiveConnections;
use crate::prelude::*;

pub fn broadcast_player_attributes_system(
    mut server: ResMut<RenetServer>,
    usernames: Res<ClientUsernames>,
    player_states: Res<player_resources::PlayerStates>,
    active_connections: Res<ActiveConnections>,
) {
    for client_id in server.clients_id() {
        let other_player_states: HashMap<Username, PlayerState> = player_states
            .players
            .iter()
            .filter(|(username, _)| {
                usernames
                    .get_client_id(username)
                    .is_some_and(|client_id| active_connections.is_accepted(client_id))
            })
            .filter(|(username, _)| {
                usernames
                    .username_for_client_id(&client_id)
                    .is_none_or(|own_username| own_username != *username)
            })
            .map(|(u, s)| (*u, *s))
            .collect();

        server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            bincode::serialize(&NetworkingMessage::PlayerSync(other_player_states)).unwrap(),
        );
    }
}
