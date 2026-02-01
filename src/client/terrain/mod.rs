use crate::prelude::*;

pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        info!("Building TerrainPlugin");
        app.insert_resource(ChunkManager::new());
        app.insert_resource(util::TextureManager::new());
        app.insert_resource(resources::RenderMaterials::new());
        app.insert_resource(resources::MesherTasks::default());
        app.insert_resource(resources::ChunkEntityMap::default());
        app.insert_resource(resources::RequestedChunks::default());
        app.add_message::<terrain_events::BlockUpdateEvent>();
        app.add_message::<terrain_events::ChunkMeshUpdateEvent>();
        app.add_message::<terrain_events::WorldRegenerateEvent>();
        app.add_message::<terrain_events::RerequestChunks>();
        app.add_message::<terrain_events::RequestChunkBatch>();
        app.add_message::<terrain_events::CleanupChunksAroundOrigin>();
        app.add_systems(Startup, terrain_systems::prepare_mesher_materials_system);
        #[cfg(feature = "skip_terrain")]
        {
            app.insert_resource(terrain_resources::SpawnRegionLoaded(true));
            app.add_systems(Startup, terrain_systems::generate_simple_ground_system);
        }
        #[cfg(not(feature = "skip_terrain"))]
        {
            app.insert_resource(terrain_resources::SpawnRegionLoaded(false));

            app.add_systems(
                OnEnter(GameState::LoadingSpawnRegion),
                terrain_systems::generate_world_system,
            );
            app.add_systems(
                Update,
                (terrain_systems::check_if_spawn_area_is_loaded_system)
                    .run_if(in_state(GameState::LoadingSpawnRegion)),
            );
            app.add_systems(
                Update,
                terrain_systems::handle_chunk_mesh_update_events_system,
            );
            app.add_systems(
                Update,
                terrain_systems::handle_terrain_regeneration_events_system,
            );
            app.add_systems(Update, terrain_systems::handle_chunk_tasks_system);
            app.add_systems(Update, terrain_systems::handle_chunk_rerequests_system);
            app.add_systems(
                Update,
                terrain_systems::handle_chunk_request_chunk_batch_event_system,
            );
            app.add_systems(Update, terrain_systems::cleanup_chunk_entities_system);
        }
    }
}
