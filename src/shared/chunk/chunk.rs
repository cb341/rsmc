use bevy::math::IVec3;

use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub data: [BlockId; CHUNK_LENGTH],
    pub position: IVec3,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new(IVec3::ZERO)
    }
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            data: [BlockId::Air; CHUNK_LENGTH],
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
        let n  = PADDED_CHUNK_SIZE;
        assert!(x <  n && y < n && z < n, "Index out of bounds: ({}, {}, {})", x,y,z);

        x + n * (y + n * z)
    }

    pub fn key_eq_pos(key: [i32; 3], position: IVec3) -> bool {
        position.x == key[0] && position.y == key[1] && position.z == key[2]
    }
}
