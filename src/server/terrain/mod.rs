use crate::{prelude::*, terrain::persistence::WorldSave};
use rand::distr::{Alphanumeric, SampleString};

pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

mod persistence;

pub enum TerrainStrategy {
    SeededRandom(u32),
    LoadFromSave(Box<WorldSave>),
}

pub struct TerrainPlugin {
    strategy: TerrainStrategy,
}

impl TerrainPlugin {
    pub fn from_world_name(world_name: &String) -> std::result::Result<Self, std::io::Error> {
        println!("Loading world '{}'", world_name);
        let world_save = persistence::read_world_save_by_name(world_name).map_err(|err| {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("Error: Save File not found '{}'. Make sure it is located within 'worlds/' directory", world_name)
                }
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!(
                        "Error: Permission denied. Check file permissions '{}'.",
                        world_name
                    )
                }
                _ => eprintln!("Unknown Error loading file: {}", err),
            }
            err
        })?;

        Ok(Self {
            strategy: TerrainStrategy::LoadFromSave(Box::new(world_save)),
        })
    }

    pub fn from_seed(seed: u32) -> TerrainPlugin {
        Self {
            strategy: TerrainStrategy::SeededRandom(seed),
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        match &self.strategy {
            TerrainStrategy::SeededRandom(seed) => {
                let random_name = Alphanumeric.sample_string(&mut rand::rng(), 16);
                println!(
                    "Generating new world '{}' with seed [{}]",
                    random_name, seed
                );

                app.insert_resource(resources::AutoSaveName::with_name(random_name));
                app.insert_resource(ChunkManager::new());
                app.insert_resource(resources::Generator::with_seed(*seed));
                app.add_systems(Startup, terrain_systems::setup_world_system);
            }
            TerrainStrategy::LoadFromSave(world_save) => {
                app.insert_resource(resources::AutoSaveName::with_name(world_save.name.clone()));
                app.insert_resource(ChunkManager::with_chunks(world_save.chunks.clone()));
                app.insert_resource(world_save.generator.clone());
            }
        }

        app.add_message::<terrain_events::BlockUpdateEvent>();
        app.insert_resource(resources::PastBlockUpdates::default());
        app.insert_resource(resources::WorldBackupTimer::default());
        app.insert_resource(resources::WorldSaveTimer::default());
        app.add_systems(Update, terrain_systems::process_user_chunk_requests_system);
        app.add_systems(Update, terrain_systems::save_world_system);
        app.add_systems(Update, terrain_systems::backup_world_system);
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
