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

    pub fn is_accepted(&self, client_id: &ClientId) -> bool {
        self.accepted_clients.contains(client_id)
    }
}

#[derive(Default, Resource)]
pub struct PendingDisconnects {
    pending: Vec<ClientId>,
    ready: Vec<ClientId>,
}

impl PendingDisconnects {
    pub fn queue(&mut self, client_id: ClientId) {
        self.pending.push(client_id);
    }

    pub fn drain_ready(&mut self) -> Vec<ClientId> {
        let to_disconnect = std::mem::take(&mut self.ready);
        self.ready = std::mem::take(&mut self.pending);
        to_disconnect
    }
}
