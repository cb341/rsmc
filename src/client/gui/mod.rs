pub mod components;
pub mod events;
pub mod systems;

use bevy::{prelude::*, text::FontSmoothing};
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};

#[cfg(feature = "debug_ui")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, gui_systems::setup_gui_system);
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 16.0,
                    font: default(),
                    font_smoothing: FontSmoothing::default(),
                    ..default()
                },
                text_color: Color::srgb(0.0, 1.0, 0.0),
                refresh_interval: core::time::Duration::from_millis(10),
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: true,
                    min_fps: 30.0,
                    target_fps: 120.0,
                },
            },
        });

        #[cfg(feature = "debug_ui")]
        {
            app.add_plugins(WorldInspectorPlugin::default());
            app.add_systems(Update, gui_systems::handle_debug_state_transition_system);

            app.add_systems(
                OnEnter(GameState::Debugging),
                gui_systems::handle_enter_debug_state_system,
            );
        }
    }
}
