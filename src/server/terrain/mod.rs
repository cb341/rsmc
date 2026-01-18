use crate::{prelude::*, terrain::persistence::WorldSave};

pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

mod persistence;

pub enum TerrainStrategy {
    SeededRandom(u32),
    LoadFromSave(WorldSave),
}

pub struct TerrainPlugin {
    strategy: TerrainStrategy,
}

impl TerrainPlugin {
    pub fn from_path(file_path: &String) -> std::result::Result<Self, std::io::Error> {
        println!("Loading world save from file '{}'", file_path);
        let world_save = match persistence::read_world_save_from_disk(file_path) {
            Ok(world_save) => world_save,
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => eprintln!("Error: Save File not found '{}'", file_path),
                    std::io::ErrorKind::PermissionDenied => eprintln!("Error: Permission denied. Check file permissions."),
                    std::io::ErrorKind::StorageFull => eprintln!("Error: Not enough disk space to save."),
                    _ => eprintln!("Unknown Error saving file: {}", err),
                }
                return Err(err);
            }
        };

        Ok(Self {
            strategy: TerrainStrategy::LoadFromSave(world_save)
        })
    }

    pub fn from_seed(seed: u32) -> TerrainPlugin {
        Self {
            strategy: TerrainStrategy::SeededRandom(seed)
        }
    }
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
            TerrainStrategy::LoadFromSave(world_save) => {
                let mut manager = ChunkManager::new();
                manager.insert_chunks(world_save.chunks.clone());
                app.insert_resource(manager);
                app.insert_resource(world_save.generator.clone());
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
