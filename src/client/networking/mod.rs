pub mod commands;
pub mod systems;

use crate::connection_config;
use bevy_renet::{
    netcode::{ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport},
    RenetClientPlugin,
};

use crate::prelude::*;

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:5000";

pub struct NetworkingPlugin {
    username: Username,
    server_addr: SocketAddr,
}

impl NetworkingPlugin {
    pub fn new(server_addr: &str, username: String) -> Result<NetworkingPlugin, String> {
        let server_addr = server_addr.parse().map_err(|_| {
            format!(
                "Address '{}' is invalid, please specify address in format like 127.0.0.1:500",
                server_addr
            )
        })?;

        Ok(Self {
            server_addr,
            username: Username(username),
        })
    }
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));

        let client = RenetClient::new(connection_config());
        app.insert_resource(client);

        let authentication = ClientAuthentication::Unsecure {
            server_addr: self.server_addr,
            client_id: rand::random::<u64>(),
            user_data: Some(self.username.to_netcode_user_data()),
            protocol_id: 0,
        };
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
        app.insert_resource(transport);

        app.add_systems(Update, networking_systems::receive_message_system);
    }
}
