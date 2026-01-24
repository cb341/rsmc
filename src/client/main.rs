use crate::prelude::*;

pub mod prelude;

mod chat;
mod collider;
mod gui;
mod networking;
mod player;
mod remote_player;
mod scene;
mod states;
mod terrain;

use bevy_flair::FlairPlugin;
use clap::{command, Parser};
use scene::setup_scene;

#[cfg(feature = "wireframe")]
mod wireframe_config {
    use crate::wireframe::{WireframeConfig, WireframePlugin};
    use bevy::color::palettes::css::WHITE;

    pub fn wireframe_plugin() -> WireframePlugin {
        WireframePlugin::default()
    }

    pub fn wireframe_config() -> WireframeConfig {
        WireframeConfig {
            global: true,
            default_color: WHITE.into(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(version)]
#[command(long_about = None)]
struct Cli {
    #[command(flatten)]
    networking_args: networking_commands::NetworkingArgs,
}

fn main() {
    let cli = Cli::parse();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(2.0),
            present_mode: bevy::window::PresentMode::Immediate,
            ..default()
        }),
        ..default()
    };

    let default_plugins = DefaultPlugins
        .set(window_plugin)
        .set(ImagePlugin::default_nearest());

    let mut app = App::new();

    let networking_plugin = networking::NetworkingPlugin::from_args(cli.networking_args);
    match networking_plugin {
        Ok(plugin) => {
            app.add_plugins(plugin);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    }

    app.add_plugins((
        default_plugins,
        FlairPlugin,
        #[cfg(feature = "wireframe")]
        wireframe_config::wireframe_plugin(),
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
        gui::GuiPlugin,
        terrain::TerrainPlugin,
        collider::ColliderPlugin,
        player::PlayerPlugin,
        remote_player::RemotePlayerPlugin,
        #[cfg(feature = "chat")]
        chat::ChatPlugin,
    ));
    app.insert_state(GameState::Playing);

    #[cfg(feature = "wireframe")]
    app.insert_resource(wireframe_config::wireframe_config());

    app.add_systems(Startup, setup_scene).run();
}
