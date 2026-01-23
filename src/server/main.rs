pub mod chat;
pub mod networking;
pub mod player;
pub mod prelude;
pub mod terrain;

use clap::{arg, command, Parser, Subcommand};

#[cfg(feature = "egui_layer")]
use bevy::DefaultPlugins;
#[cfg(feature = "egui_layer")]
pub mod gui;

#[cfg(not(feature = "egui_layer"))]
use bevy::log::LogPlugin;
use rand::RngCore;

use crate::prelude::*;

#[derive(Debug, Subcommand)]
enum Commands {
    GenerateWorld {
        #[arg(required = true)]
        world_name: String,
        #[arg(short, long = "replace", help = "Replace existing")]
        replace_existing: bool,
        #[arg(short, long)]
        seed: Option<u32>,
    },
    LoadWorld {
        #[arg()]
        world_name: String,
    },
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    let terrain_plugin = match args.command {
        Commands::GenerateWorld {
            world_name,
            replace_existing,
            seed,
        } => {
            let seed = seed.unwrap_or_else(|| rand::rng().next_u32());
            terrain::TerrainPlugin::new_with_seed(world_name, replace_existing, seed)
        }
        Commands::LoadWorld { world_name } => terrain::TerrainPlugin::load_from_save(&world_name),
    };

    match terrain_plugin {
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
