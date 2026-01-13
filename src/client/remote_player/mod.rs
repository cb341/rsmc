pub mod components;
pub mod events;
pub mod systems;

use crate::prelude::*;

pub struct RemotePlayerPlugin;

impl Plugin for RemotePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<events::RemotePlayerSpawnedEvent>();
        app.init_gizmo_group::<remote_player_components::RemotePlayerGizmos>();
        app.add_message::<events::RemotePlayerDespawnedEvent>();
        app.add_message::<events::RemotePlayerSyncEvent>();
        app.add_systems(
            Update,
            (
                remote_player_systems::spawn_remote_player_system,
                remote_player_systems::update_remote_player_system,
                remote_player_systems::despawn_remote_player_system,
                remote_player_systems::draw_gizmos,
            ),
        );
    }
}
