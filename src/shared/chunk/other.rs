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
