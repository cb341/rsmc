use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    fmt::Display,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{prelude::*, terrain::resources::Generator};

const BACKUPS_DIR: &str = "backups/";
const WORLDS_DIR: &str = "worlds/";
const WORLD_EXTENSION: &str = ".rsmcw";

#[derive(Serialize, Deserialize, Default)]
pub struct WorldSave {
    pub name: String,
    pub generator: Generator,
    pub chunks: Vec<Chunk>,
}

impl Display for WorldSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{} chunks]", self.name, self.chunks.len())
    }
}

impl WorldSave {
    fn backup_path(&self) -> PathBuf {
        path_for_world_backup(&self.name, Utc::now())
    }

    fn save_path(&self) -> PathBuf {
        path_for_world(&self.name)
    }
}

fn path_for_world(world_name: &str) -> PathBuf {
    let file_name = format!("{}{}", world_name, WORLD_EXTENSION);
    PathBuf::from(WORLDS_DIR).join(file_name)
}

fn path_for_world_backup(world_name: &str, timestamp: DateTime<Utc>) -> PathBuf {
    let file_name = format!(
        "{}_{}.rsmcw.bak",
        world_name,
        timestamp.format("%Y%m%d%H%M%S%3f")
    );
    PathBuf::from(BACKUPS_DIR).join(file_name)
}

fn upsert_file(world_save: &WorldSave, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    let serialized = bincode::serialize(world_save)?;
    file.write_all(&serialized)?;
    file.flush()?;

    Ok(())
}

fn build_world_save_from_resources(
    name: &str,
    chunk_manager: &ChunkManager,
    generator: &Generator,
) -> WorldSave {
    let chunks = chunk_manager.all_chunks().into_iter().copied().collect();
    let generator = generator.clone();

    WorldSave {
        name: String::from(name),
        generator,
        chunks,
    }
}

pub fn save_world(
    name: &str,
    chunk_manager: &ChunkManager,
    generator: &Generator,
) -> Result<(), Box<dyn std::error::Error>> {
    let world_save = build_world_save_from_resources(name, chunk_manager, generator);
    update_world_file(&world_save)
}

pub fn backup_world(
    name: &str,
    chunk_manager: &ChunkManager,
    generator: &Generator,
) -> Result<(), Box<dyn std::error::Error>> {
    let world_save = build_world_save_from_resources(name, chunk_manager, generator);
    create_backup(&world_save)
}

fn create_backup(world_save: &WorldSave) -> Result<(), Box<dyn std::error::Error>> {
    let path = world_save.backup_path();
    upsert_file(world_save, &path)?;
    println!("Saved world backup to: '{}'", path.display());
    Ok(())
}

fn update_world_file(world_save: &WorldSave) -> Result<(), Box<dyn std::error::Error>> {
    let path = world_save.save_path();
    upsert_file(world_save, &path)?;
    println!("Updated world file: '{}'", path.display());
    Ok(())
}

pub use ecs_api::*;

pub mod ecs_api {
    use super::*;

    pub fn read_world_from_name(name: &str) -> Result<WorldSave, std::io::Error> {
        let path = path_for_world(name);
        read_world_save_from_disk(&path)
    }

    fn read_world_save_from_disk(path: &Path) -> Result<WorldSave, std::io::Error> {
        use std::io::Read;
        let mut file = File::open(path)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("File data is supposed to be readable");
        let world_save: WorldSave =
        bincode::deserialize(&buffer).expect("World Save is expected to be deserializable");

        Ok(world_save)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_read_generated_world_from_disk() {
        let mut generator = Generator::with_seed(0);
        generator.params.density.squash_factor = 6.7;

        let mut chunk_manager = ChunkManager::new();
        let mut chunks = ChunkManager::instantiate_chunks(IVec3::ZERO, IVec3::ONE);

        assert!(!chunks.is_empty());

        chunks.par_iter_mut().for_each(|chunk| {
            generator.generate_chunk(chunk);
        });

        chunk_manager.insert_chunks(chunks);
        save_world("my_world", &chunk_manager, &generator).unwrap();

        let world = read_world_from_name("my_world").unwrap();

        assert!(!world.chunks.is_empty());
        assert_eq!(world.name, "my_world");
        assert_eq!(world.generator.params.density.squash_factor, 6.7);
    }
}
