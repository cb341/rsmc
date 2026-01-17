use serde::Serialize;
use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::{prelude::*, terrain::resources::Generator};

const WORLDS_DIR: &str = "backups/";
const SAVE_VERSION: &str = "0.1";

#[derive(Serialize, Deserialize)]
struct WorldSave {
    pub name: String,
    pub version: String,
    pub generator: Generator,
    pub chunks: Vec<Chunk>,
}

impl Display for WorldSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.name, self.version)
    }
}

impl WorldSave {
    fn name(generation: usize) -> String {
        format!("world_backup_{}.rsmcw", generation.to_string())
    }
}

fn save_world_to_file(world_save: WorldSave) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(WORLDS_DIR)?;

    let file_path_str: &str = &(String::from(WORLDS_DIR) + &world_save.name);

    let path = Path::new(file_path_str);
    let mut file = File::create(path)?;
    let serialized = bincode::serialize(&world_save)?;
    file.write_all(&serialized)?;
    // let serialized = serde_json::to_string(&world_save)?;
    // file.write_all(&serialized.into_bytes())?;

    Ok(())
}

pub fn save_world_to_disk(generation: usize, chunk_manager: &ChunkManager, generator: &Generator) {
    let chunks = chunk_manager
        .all_chunks()
        .into_iter()
        .map(|v| v.clone())
        .collect();
    let generator = generator.clone();

    let world_save = WorldSave {
        name: WorldSave::name(generation),
        version: String::from(SAVE_VERSION),
        generator,
        chunks,
    };

    match save_world_to_file(world_save) {
        Ok(_) => info!("Saved World!"),
        Err(err) => error!("Error occured saving world: {}", err),
    };
}

pub fn read_world_save_from_disk(path: String) -> Result<WorldSave, Box<dyn std::error::Error>> {
    // TODO: test
    let mut file = File::open(Path::new(&path))?;
    let bytes = file.bytes();
    let bytes: Bytes = bytes.into();

    let world_save: WorldSave = bincode::deserialize(&bytes)?;

    Ok(world_save)
}
