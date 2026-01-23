use crate::{prelude::*, terrain::persistence::WorldSave};

pub mod events;
pub mod resources;
pub mod systems;
pub mod util;

mod persistence;

pub enum TerrainStrategy {
    SeededRandom(String, u32),
    LoadFromSave(Box<WorldSave>),
}

pub struct TerrainPlugin {
    strategy: TerrainStrategy,
}

impl TerrainPlugin {
    pub fn load_from_save(world_name: &str) -> Result<Self, String> {
        println!("Loading world '{}'...", world_name);
        let world_save = persistence::read_world_save_by_name(world_name).map_err(|err| {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    format!("Save File '{}' not found. Make sure it is located within 'worlds/' directory", world_name)
                }
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied. Check file permissions '{}'.",
                        world_name
                    )
                }
                _ => format!("Unknown Error loading file: {}", err)
            }
        })?;

        Ok(Self {
            strategy: TerrainStrategy::LoadFromSave(Box::new(world_save)),
        })
    }

    pub fn new_with_seed(world_name: String, replace: bool, seed: u32) -> Result<Self, String> {
        if !replace && persistence::world_save_exists(&world_name) {
            Err(format!(
                "World Save '{}' already exists, pass replace flag if you want to replace it",
                world_name
            ))
        } else {
            Ok(Self {
                strategy: TerrainStrategy::SeededRandom(world_name, seed),
            })
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        match &self.strategy {
            TerrainStrategy::SeededRandom(world_name, seed) => {
                println!("Generating new world '{}' with seed [{}]", world_name, seed);

                app.insert_resource(resources::AutoSaveName::with_name(world_name.clone()));
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
