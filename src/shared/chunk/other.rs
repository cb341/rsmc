use std::collections::HashMap;

use bevy::{math::Vec3, prelude::Resource};

use crate::*;

impl Chunk {
    pub fn new(position: Vec3) -> Self {
        Self {
            data: [BlockId::Air; CHUNK_LENGTH],
            position,
        }
    }

    pub fn valid_padded(x: usize, y: usize, z: usize) -> bool {
        (0..CHUNK_SIZE).contains(&x) && (0..CHUNK_SIZE).contains(&y) && (0..CHUNK_SIZE).contains(&z)
    }

    pub fn valid_unpadded(x: usize, y: usize, z: usize) -> bool {
        x < PADDED_CHUNK_SIZE && y < PADDED_CHUNK_SIZE && z < PADDED_CHUNK_SIZE
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.get_unpadded(x + 1, y + 1, z + 1)
    }

    pub fn get_unpadded(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.data[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: BlockId) {
        self.set_unpadded(x + 1, y + 1, z + 1, value);
    }

    pub fn update(&mut self, x: usize, y: usize, z: usize, value: BlockId) {
        self.set(x, y, z, value);

        if !value.supports_grass()
            && Self::valid_padded(x, y + 1, z)
            && self.get(x, y + 1, z) == BlockId::Tallgrass
        {
            self.set(x, y + 1, z, BlockId::Air);
        }
    }

    pub fn set_unpadded(&mut self, x: usize, y: usize, z: usize, value: BlockId) {
        self.data[Self::index(x, y, z)] = value;
    }

    #[rustfmt::skip]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        if (x >= PADDED_CHUNK_SIZE) || (y >= PADDED_CHUNK_SIZE) || (z >= PADDED_CHUNK_SIZE) {
            panic!("Index out of bounds: ({}, {}, {})", x, y, z);
        }
        x + PADDED_CHUNK_USIZE * (y + PADDED_CHUNK_USIZE * z)
    }

    pub fn key_eq_pos(key: [i32; 3], position: Vec3) -> bool {
        position.x as i32 == key[0] && position.y as i32 == key[1] && position.z as i32 == key[2]
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
