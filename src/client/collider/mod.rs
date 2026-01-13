pub mod components;
pub mod events;
pub mod systems;

use crate::prelude::*;

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, collider_systems::setup_coliders_system);
        app.add_message::<collider_events::ColliderUpdateEvent>();
        app.add_systems(
            Update,
            collider_systems::handle_collider_update_events_system,
        );
    }
}
