pub mod chat;
pub mod networking;
pub mod player;
pub mod prelude;
pub mod terrain;

use clap::{command, Parser};

#[cfg(feature = "egui_layer")]
use bevy::DefaultPlugins;
#[cfg(feature = "egui_layer")]
pub mod gui;

#[cfg(not(feature = "egui_layer"))]
use bevy::log::LogPlugin;

use crate::prelude::*;

#[derive(Debug, Parser)]
#[command(version)]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    world_commands: terrain_commands::WorldCommands,
}

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

    let args = Cli::parse();
    match terrain::TerrainPlugin::from_command(args.world_commands) {
        Ok(terrain_plugin) => app.add_plugins(terrain_plugin),
        Err(error) => {
            eprintln!("Error: {}", error);
            return;
        }
    };

    app.add_plugins(player::PlayerPlugin);
    app.add_plugins(networking::NetworkingPlugin);

    #[cfg(feature = "chat")]
    app.add_plugins(chat::ChatPlugin);

    println!("Server is starting!");
    app.run();
}
