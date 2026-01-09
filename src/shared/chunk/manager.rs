use std::collections::HashMap;

use bevy::{math::Vec3, prelude::Resource};

use crate::*;

#[derive(Resource)]
pub struct ChunkManager {
    pub chunks: HashMap<[i32; 3], Chunk>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn instantiate_chunks(position: Vec3, render_distance: Vec3) -> Vec<Chunk> {
        let render_distance_x = render_distance.x as i32;
        let render_distance_y = render_distance.y as i32;
        let render_distance_z = render_distance.z as i32;

        let mut chunks: Vec<Chunk> = Vec::new();

        for x in -render_distance_x..render_distance_x {
            for y in -render_distance_y..render_distance_y {
                for z in -render_distance_z..render_distance_z {
                    let chunk_position = Vec3::new(
                        x as f32 + position.x,
                        y as f32 + position.y,
                        z as f32 + position.z,
                    );
                    let chunk = Chunk::new(chunk_position);
                    chunks.push(chunk);
                }
            }
        }

        chunks
    }

    pub fn instantiate_new_chunks(&mut self, position: Vec3, render_distance: Vec3) -> Vec<Chunk> {
        let chunks = Self::instantiate_chunks(position, render_distance);

        chunks
            .into_iter()
            .filter(|chunk| {
                let chunk_position = chunk.position;
                let chunk = self.get_chunk_mut(chunk_position);
                chunk.is_none()
            })
            .collect()
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks
            .insert(Self::position_to_key(chunk.position), chunk);
    }

    pub fn insert_chunks(&mut self, chunks: Vec<Chunk>) {
        for chunk in chunks {
            self.insert_chunk(chunk);
        }
    }

    pub fn position_to_key(position: Vec3) -> [i32; 3] {
        [position.x as i32, position.y as i32, position.z as i32]
    }

    pub fn set_chunk(&mut self, position: Vec3, chunk: Chunk) {
        let Vec3 { x, y, z } = position;

        self.chunks.insert([x as i32, y as i32, z as i32], chunk);
    }

    pub fn get_chunk(&self, position: Vec3) -> Option<&Chunk> {
        let Vec3 { x, y, z } = position.floor();

        self.chunks.get(&[x as i32, y as i32, z as i32])
    }

    pub fn get_chunk_mut(&mut self, position: Vec3) -> Option<&mut Chunk> {
        let Vec3 { x, y, z } = position.floor();

        self.chunks.get_mut(&[x as i32, y as i32, z as i32])
    }

    pub fn update_block(&mut self, position: Vec3, block: BlockId) {
        match self.chunk_from_selection(position) {
            Some(chunk) => {
                let chunk_position = Vec3::new(
                    chunk.position[0] * CHUNK_SIZE as f32,
                    chunk.position[1] * CHUNK_SIZE as f32,
                    chunk.position[2] * CHUNK_SIZE as f32,
                );
                let local_position = (position - chunk_position).floor();
                chunk.update(
                    local_position.x as usize,
                    local_position.y as usize,
                    local_position.z as usize,
                    block,
                );
            }
            None => {
                println!("No chunk found");
            }
        }
    }

    pub fn get_block(&mut self, position: Vec3) -> Option<BlockId> {
        match self.chunk_from_selection(position) {
            Some(chunk) => {
                let chunk_position = Vec3::new(
                    chunk.position[0] * CHUNK_SIZE as f32,
                    chunk.position[1] * CHUNK_SIZE as f32,
                    chunk.position[2] * CHUNK_SIZE as f32,
                );
                let local_position = (position - chunk_position).floor();
                Some(chunk.get(
                    local_position.x as usize,
                    local_position.y as usize,
                    local_position.z as usize,
                ))
            }
            None => {
                // println!("No chunk found for block at {:?}", position);
                None
            }
        }
    }

    fn chunk_from_selection(&mut self, position: Vec3) -> Option<&mut Chunk> {
        let chunk_position = position / CHUNK_SIZE as f32;
        self.get_chunk_mut(chunk_position)
    }

    pub fn get_all_chunk_positions(&self) -> Vec<Vec3> {
        self.chunks
            .keys()
            .map(|key| Vec3::new(key[0] as f32, key[1] as f32, key[2] as f32))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_manager_new() {
        let chunk_manager = ChunkManager::new();
        assert!(chunk_manager.chunks.is_empty());
    }

    #[test]
    fn test_instantiate_chunks() {
        let position = Vec3::new(0.0, 0.0, 0.0);

        let width = 2;
        let height = 3;
        let depth = 4;

        let render_distance = Vec3::new(width as f32, height as f32, depth as f32);

        let chunks = ChunkManager::instantiate_chunks(position, render_distance);
        assert_eq!(chunks.len(), (2 * width * 2 * height * 2 * depth) as usize,);
    }

    #[test]
    fn test_insert_chunks() {
        let mut chunk_manager = ChunkManager::new();
        let position = Vec3::new(0.0, 0.0, 0.0);
        let render_distance = 2;
        let chunks = ChunkManager::instantiate_chunks(
            position,
            Vec3::new(
                render_distance as f32,
                render_distance as f32,
                render_distance as f32,
            ),
        );

        let render_diameter = render_distance * 2;

        chunk_manager.insert_chunks(chunks);
        assert_eq!(
            chunk_manager.chunks.len(),
            (render_diameter * render_diameter * render_diameter) as usize
        );
    }

    #[test]
    fn test_set_and_get_chunk_mut() {
        let mut chunk_manager = ChunkManager::new();
        let position = Vec3::new(0.0, 0.0, 0.0);
        let chunk = Chunk::new(position);

        chunk_manager.set_chunk(position, chunk);
        let retrieved_chunk = chunk_manager.get_chunk_mut(position).unwrap();
        assert_eq!(retrieved_chunk.position, chunk.position);
    }

    #[test]
    fn test_set_and_get_block() {
        let mut chunk_manager = ChunkManager::new();
        let position = Vec3::new(0.0, 0.0, 0.0);
        let chunk = Chunk::new(position);

        chunk_manager.set_chunk(position, chunk);
        let block_position = Vec3::new(1.0, 1.0, 1.0);
        let block_id = BlockId::Stone;

        chunk_manager.update_block(block_position, block_id);
        let retrieved_block = chunk_manager.get_block(block_position).unwrap();
        assert_eq!(retrieved_block, block_id);
    }

    #[test]
    fn test_get_all_chunk_positions() {
        let mut chunk_manager = ChunkManager::new();
        chunk_manager.set_chunk(Vec3::new(0.0, 0.0, 0.0), Chunk::default());
        chunk_manager.set_chunk(Vec3::new(2.0, 0.0, 0.0), Chunk::default());
        chunk_manager.set_chunk(Vec3::new(1.0, 0.0, 3.0), Chunk::default());

        let retrieved_chunk_positions = chunk_manager.get_all_chunk_positions();
        assert_eq!(retrieved_chunk_positions.len(), 3);
    }

    #[test]
    #[rustfmt::skip]
    fn test_tallgrass_update() {
        let mut chunk_manager = ChunkManager::new();
        let chunk_position = Vec3::new(0.0, 0.0, 0.0);
        let chunk = Chunk::new(chunk_position);
        chunk_manager.set_chunk(chunk_position, chunk);

        let grass_position = Vec3::new(0.0, 0.0, 0.0);
        let tallgrass_position = Vec3::new(0.0, 1.0, 0.0);

        chunk_manager.update_block(grass_position, BlockId::Grass);
        assert_eq!(chunk_manager.get_block(grass_position).unwrap(), BlockId::Grass);
        chunk_manager.update_block(tallgrass_position, BlockId::Tallgrass);
        assert_eq!(chunk_manager.get_block(tallgrass_position).unwrap(), BlockId::Tallgrass);

        chunk_manager.update_block(grass_position, BlockId::Dirt);
        assert_eq!(chunk_manager.get_block(grass_position).unwrap(), BlockId::Dirt);
        assert_eq!(chunk_manager.get_block(tallgrass_position).unwrap(), BlockId::Tallgrass);

        chunk_manager.update_block(grass_position, BlockId::Air);
        assert_eq!(chunk_manager.get_block(grass_position).unwrap(), BlockId::Air);
        assert_eq!(chunk_manager.get_block(tallgrass_position).unwrap(), BlockId::Air);
    }
}
