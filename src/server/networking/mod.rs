pub mod systems;

use crate::connection_config;

use crate::prelude::*;

const SERVER_ADDR: &str = "127.0.0.1:5000";

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin);

        #[cfg(feature = "renet_visualizer")]
        {
            use bevy_egui::EguiPrimaryContextPass;
            use renet_visualizer::RenetServerVisualizer;

            app.insert_resource(RenetServerVisualizer::<200>::default());
            app.add_systems(
                Update,
                (networking_systems::handle_events_for_visualizer_system,),
            );
            app.add_systems(
                EguiPrimaryContextPass,
                (networking_systems::update_visulizer_system,),
            );
        }

        info!("Config: {:?}", connection_config());

        let server = RenetServer::new(connection_config());

        app.insert_resource(server);

        app.add_plugins(NetcodeServerPlugin);
        let server_addr = SERVER_ADDR.parse().unwrap();
        let socket = UdpSocket::bind(server_addr).unwrap();
        let server_config = ServerConfig {
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            max_clients: 64,
            protocol_id: 0,
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,
        };
        let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
        app.insert_resource(transport);

        app.add_systems(Update, networking_systems::receive_message_system);
        app.add_systems(Update, networking_systems::handle_events_system);
    }
}
