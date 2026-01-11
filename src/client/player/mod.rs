pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

use crate::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        debug!("Building PlayerPlugin");
        info!("Building PlayerPlugin");
        app.add_plugins(FpsControllerPlugin);
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        #[cfg(feature = "physics_debug")]
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.add_event::<player_events::PlayerColliderUpdateEvent>();
        app.insert_resource(player_resources::BlockSelection::new());
        app.insert_resource(player_resources::PlayerSpawned(false));
        app.insert_resource(player_resources::LastPlayerPosition::new());
        app.add_systems(
            Startup,
            (
                player_systems::setup_highlight_cube_system,
                player_systems::setup_player_camera,
            ),
        );
        app.add_systems(
            Update,
            (player_systems::setup_controller_on_area_ready_system,)
                .run_if(terrain_resources::SpawnAreaLoaded::is_loaded)
                .run_if(player_resources::PlayerSpawned::is_not_spawned),
        );
        app.add_systems(
            Update,
            (
                player_systems::handle_controller_movement_system,
                player_systems::handle_player_collider_events_system,
            )
                .run_if(player_resources::PlayerSpawned::is_spawned),
        );
        app.add_systems(
            Update,
            (
                player_systems::manage_cursor_system,
                player_systems::handle_mouse_events_system,
                player_systems::handle_keyboard_events_system,
                player_systems::raycast_system,
                player_systems::handle_block_update_events,
                player_systems::broadcast_player_attributes_system,
            )
                .run_if(player_resources::PlayerSpawned::is_spawned)
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            OnEnter(GameState::Playing),
            (
                player_systems::activate_fps_controller_system,
                player_systems::lock_cursor_system,
            ),
        );

        app.add_systems(
            OnExit(GameState::Playing),
            player_systems::deactivate_fps_controller_system,
        );
    }
}
