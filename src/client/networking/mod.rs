pub mod systems;

use crate::connection_config;
use bevy_renet::{
    RenetClientPlugin,
    netcode::{ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport},
};

use crate::prelude::*;

const SERVER_ADDR: &str = "127.0.0.1:5000";

pub struct NetworkingPlugin;
impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));

        let client = RenetClient::new(connection_config());
        app.insert_resource(client);

        let client_id = rand::random::<u64>();
        let authentication = ClientAuthentication::Unsecure {
            server_addr: SERVER_ADDR.parse().unwrap(),
            client_id,
            user_data: None,
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
