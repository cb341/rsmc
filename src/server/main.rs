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

    let file_path: Option<String> = None;

    let terrain_strategy = match file_path {
        Some(file_path) => terrain::TerrainStrategy::LoadFromFile(String::from(file_path)),
        None => terrain::TerrainStrategy::SeededRandom(0)
    };

    app.add_plugins(terrain::TerrainPlugin {
        strategy: terrain_strategy
    });

    #[cfg(feature = "chat")]
    app.add_plugins(chat::ChatPlugin);

    app.run();
}
