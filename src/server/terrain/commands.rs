use clap::{arg, Subcommand};
use rand::RngCore;

use crate::terrain::TerrainPlugin;

#[derive(Debug, Subcommand)]
pub enum WorldCommands {
    #[command(about = "Generate a new world with the given name")]
    GenerateWorld {
        #[arg(required = true)]
        world_name: String,
        #[arg(short, long = "replace", help = "Replace existing")]
        replace_existing: bool,
        #[arg(short, long, help = "Seed for world generation")]
        seed: Option<u32>,
    },
    #[command(about = "Load an existing world from disk")]
    LoadWorld {
        #[arg()]
        world_name: String,
    },
}

impl TerrainPlugin {
    pub fn from_command(command: WorldCommands) -> Result<TerrainPlugin, String> {
        match command {
            WorldCommands::GenerateWorld {
                world_name,
                replace_existing,
                seed,
            } => {
                let seed = seed.unwrap_or_else(|| rand::rng().next_u32());
                Self::new_with_seed(world_name, replace_existing, seed)
            }
            WorldCommands::LoadWorld { world_name } => Self::load_from_save(&world_name),
        }
    }
}
