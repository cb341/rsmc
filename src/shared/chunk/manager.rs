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
