pub mod chat;
pub mod networking;
pub mod player;
pub mod prelude;
pub mod terrain;

use clap::Parser;

#[cfg(feature = "egui_layer")]
use bevy::DefaultPlugins;
#[cfg(feature = "egui_layer")]
pub mod gui;

#[cfg(not(feature = "egui_layer"))]
use bevy::log::LogPlugin;

use crate::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = None)]
    world_name: Option<String>,
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

    let args = Args::parse();

    let terrain_plugin = match args.world_name {
        Some(world_name) => terrain::TerrainPlugin::from_world_name(&world_name),
        None => Ok(terrain::TerrainPlugin::from_seed(0))
    };

    match terrain_plugin {
        Ok(terrain_plugin) => {app.add_plugins(terrain_plugin);},
        Err(error) => { eprintln!("Error: {}", error); return }
    };

    app.add_plugins(player::PlayerPlugin);
    app.add_plugins(networking::NetworkingPlugin);

    #[cfg(feature = "chat")]
    app.add_plugins(chat::ChatPlugin);

    println!("Application is starting!");
    app.run();
}
