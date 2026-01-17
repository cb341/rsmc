use crate::prelude::*;

pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

mod persistence;

pub enum TerrainStrategy {
    SeededRandom(u32),
    LoadFromFile(String),
}

pub struct TerrainPlugin {
    pub strategy: TerrainStrategy,
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        match &self.strategy {
            TerrainStrategy::SeededRandom(seed) => {
                println!("Generating new world with seed [{}]", seed);
                app.insert_resource(ChunkManager::new());
                app.insert_resource(resources::Generator::with_seed(*seed));
                app.add_systems(Startup, terrain_systems::setup_world_system);
            }
            TerrainStrategy::LoadFromFile(file_path) => {
                println!("Loading world save from file '{}'", file_path);
                let world_save = persistence::read_world_save_from_disk(file_path);
                match world_save {
                    Ok(world_save) => {
                        let mut manager = ChunkManager::new();
                        manager.insert_chunks(world_save.chunks.clone());
                        app.insert_resource(manager);
                        app.insert_resource(world_save.generator);
                    }
                    Err(err) => panic!("World could not be loaded! Err: {}", err),
                }
            }
        }

        app.add_message::<terrain_events::BlockUpdateEvent>();
        app.insert_resource(resources::PastBlockUpdates::default());
        app.insert_resource(resources::AutoSave::default());
        app.add_systems(Update, terrain_systems::process_user_chunk_requests_system);
        app.add_systems(Update, terrain_systems::periodic_autosave_system);
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
