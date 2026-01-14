use crate::prelude::*;

pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkManager::new());
        app.add_message::<terrain_events::BlockUpdateEvent>();
        app.insert_resource(resources::PastBlockUpdates::new());
        app.add_systems(Startup, terrain_systems::setup_world_system);
        app.add_systems(Update, terrain_systems::process_user_chunk_requests);
        app.insert_resource(resources::Generator::default());
        app.insert_resource(resources::ClientChunkRequests::default());

        #[cfg(feature = "generator_visualizer")]
        {
            app.insert_resource(resources::NoiseTextureList::default());
            app.add_systems(Startup, terrain_systems::prepare_visualizer_texture_system);
            app.add_systems(Update, terrain_systems::render_visualizer_system);
            app.add_systems(Update, terrain_systems::regenerate_heightmap_system);
            app.add_systems(Update, terrain_systems::handle_regenerate_event_system);

            app.add_message::<terrain_events::RegenerateHeightMapEvent>();
            app.add_message::<terrain_events::WorldRegenerateEvent>();
        }
    }
}
