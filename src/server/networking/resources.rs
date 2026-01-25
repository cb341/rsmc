use crate::prelude::*;

use std::collections::HashSet;

use renet::ClientId;

#[derive(Default, Resource)]
pub struct ActiveConnections {
    accepted_clients: HashSet<ClientId>,
}

impl ActiveConnections {
    pub fn accept(&mut self, client_id: ClientId) {
        self.accepted_clients.insert(client_id);
    }

    pub fn reject(&mut self, client_id: &ClientId) {
        self.accepted_clients.remove(client_id);
    }

    pub fn is_accepted(&mut self, client_id: &ClientId) -> bool {
        self.accepted_clients.contains(client_id)
    }
}
