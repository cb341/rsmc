pub mod commands;
pub mod systems;

use crate::connection_config;
use bevy_renet::{
    netcode::{
        ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport, NetcodeTransportError,
    },
    RenetClientPlugin,
};

use crate::prelude::*;

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:5000";

pub struct NetworkingPlugin {
    username: Username,
}

impl NetworkingPlugin {
    pub fn new(username: String) -> Result<NetworkingPlugin, String> {
        Ok(Self {
            username: Username::new(&username)?,
        })
    }
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));

        let client = RenetClient::new(connection_config());
        app.insert_resource(client);

        let authentication = ClientAuthentication::Unsecure {
            server_addr: DEFAULT_SERVER_ADDR
                .parse()
                .expect("Hardcoded server address should be valid"),
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

        app.add_systems(Last, networking_systems::exit_on_last_window_closed_system);
        app.add_systems(Update, networking_systems::receive_message_system);

        fn exit_on_transport_error(
            mut renet_error: MessageReader<NetcodeTransportError>,
            mut exit_events: MessageWriter<AppExit>,
        ) {
            if !renet_error.is_empty() {
                exit_events.write(AppExit::error());
            }
            for error in renet_error.read() {
                eprintln!("{}", error);
            }
        }

        app.add_systems(Update, exit_on_transport_error);
    }
}
