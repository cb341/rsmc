use serde::Serialize;
use std::{fmt::Display, fs::File, io::Write, path::Path};

use crate::{prelude::*, terrain::resources::Generator};

const WORLDS_DIR: &str = "backups/";
const SAVE_VERSION: &str = "0.1";

#[derive(Serialize, Deserialize, Default)]
pub struct WorldSave {
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
        format!("world_backup_{}.rsmcw", generation)
    }
}

fn save_world_to_file(world_save: WorldSave) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(WORLDS_DIR)?;

    let file_path_str: &str = &(String::from(WORLDS_DIR) + &world_save.name);

    let path = Path::new(file_path_str);
    let mut file = File::create(path)?;
    let serialized = bincode::serialize(&world_save)?;
    file.write_all(&serialized)?;
    file.flush()?;

    Ok(String::from(file_path_str))
}

pub fn save_world_to_disk(
    generation: usize,
    chunk_manager: &ChunkManager,
    generator: &Generator,
) -> Result<String, Box<dyn std::error::Error>> {
    let chunks = chunk_manager.all_chunks().into_iter().copied().collect();
    let generator = generator.clone();

    let world_save = WorldSave {
        name: WorldSave::name(generation),
        version: String::from(SAVE_VERSION),
        generator,
        chunks,
    };

    match save_world_to_file(world_save) {
        Ok(path) => {
            println!("Saved world backup to: '{}'", path);
            Ok(path)
        }
        Err(err) => {
            error!("Error occured saving world: {}", err);
            Err(err)
        }
    }
}

pub fn read_world_save_from_disk(path: &String) -> Result<WorldSave, std::io::Error> {
    use std::io::Read;
    let mut file = File::open(Path::new(path))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("File data is supposed to be readable");
    let world_save: WorldSave =
        bincode::deserialize(&buffer).expect("World Save is expected to be deserializable");

    Ok(world_save)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_read_world_from_disk() {
        let common_name = String::from("my_world");

        let written_save = WorldSave {
            name: common_name.clone(),
            ..Default::default()
        };

        let path = save_world_to_file(written_save).unwrap();
        let read_save = read_world_save_from_disk(&path).unwrap();
        assert!(read_save.chunks.is_empty());
        assert_eq!(read_save.name, common_name);
    }

    #[test]
    fn test_save_and_read_generated_world_from_disk() {
        let generator = Generator::with_seed(0);
        let mut chunk_manager = ChunkManager::new();
        let mut chunks = ChunkManager::instantiate_chunks(IVec3::ZERO, IVec3::ONE);

        assert!(!chunks.is_empty());

        chunks.par_iter_mut().for_each(|chunk| {
            generator.generate_chunk(chunk);
        });

        chunk_manager.insert_chunks(chunks);
        save_world_to_disk(67, &chunk_manager, &generator).unwrap();

        let world =
            read_world_save_from_disk(&(String::from("backups/") + &WorldSave::name(67))).unwrap();
        assert!(!world.chunks.is_empty());
        assert!(world.name.contains("67"));
        assert!(world.name.contains(".rsmcw"));
    }
}
