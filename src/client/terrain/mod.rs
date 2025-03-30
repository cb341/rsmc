use materials::MyExtension;

use crate::prelude::*;

pub mod components;
pub mod events;
pub mod materials;
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
        app.insert_resource(resources::MesherTasks::default());
        
        // Register our custom material extension
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MyExtension>,
        >::default());
        
        app.add_event::<terrain_events::BlockUpdateEvent>();
        app.add_event::<terrain_events::ChunkMeshUpdateEvent>();
        app.add_event::<terrain_events::WorldRegenerateEvent>();
        app.add_systems(Startup, terrain_systems::prepare_mesher_materials_system);
        #[cfg(feature = "skip_terrain")]
        {
            app.insert_resource(terrain_resources::SpawnAreaLoaded(true));
            app.add_systems(Startup, terrain_systems::generate_simple_ground_system);
        }
        #[cfg(not(feature = "skip_terrain"))]
        {
            app.insert_resource(terrain_resources::SpawnAreaLoaded(false));
            app.add_systems(Startup, terrain_systems::prepare_spawn_area_system);
            app.add_systems(Startup, terrain_systems::generate_world_system);
            app.add_systems(
                Update,
                terrain_systems::handle_chunk_mesh_update_events_system,
            );
            app.add_systems(
                Update,
                terrain_systems::handle_terrain_regeneration_events_system,
            );
            app.add_systems(Update, terrain_systems::handle_chunk_tasks_system);
        }
    }
}
