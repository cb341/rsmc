pub mod chat;
pub mod networking;
pub mod player;
pub mod prelude;
pub mod terrain;

#[cfg(feature = "egui_layer")]
use bevy::DefaultPlugins;
#[cfg(feature = "egui_layer")]
pub mod gui;

#[cfg(not(feature = "egui_layer"))]
use bevy::log::LogPlugin;

use crate::prelude::*;

fn main() {
    let mut app = App::new();
    #[cfg(not(feature = "egui_layer"))]
    {
        app.add_plugins(MinimalPlugins);
        app.add_plugins(LogPlugin::default());
    }

    #[cfg(feature = "egui_layer")]
    {
        use bevy_egui::EguiPlugin;
        app.add_plugins(DefaultPlugins);
        app.add_plugins(EguiPlugin::default());
        app.add_systems(Startup, gui::setup_camera_system);
    }

    app.add_plugins(player::PlayerPlugin);
    app.add_plugins(networking::NetworkingPlugin);
    app.add_plugins(terrain::TerrainPlugin {
        strategy: terrain::TerrainStrategy::SeededRandom(0)
    });

    #[cfg(feature = "chat")]
    app.add_plugins(chat::ChatPlugin);

    app.run();
}
