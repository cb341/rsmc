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

    pub fn valid_local(x: usize, y: usize, z: usize) -> bool {
        x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE
    }

    pub fn is_within_padded_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= -1
            && y >= -1
            && z >= -1
            && x <= CHUNK_SIZE as i32
            && y <= CHUNK_SIZE as i32
            && z <= CHUNK_SIZE as i32
    }

    pub fn valid_unpadded(x: usize, y: usize, z: usize) -> bool {
        x < PADDED_CHUNK_SIZE && y < PADDED_CHUNK_SIZE && z < PADDED_CHUNK_SIZE
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockId {
        assert!(Self::is_within_padded_bounds(x, y, z));
        self.get_unpadded((x + 1) as usize, (y + 1) as usize, (z + 1) as usize)
    }

    pub fn get_unpadded(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.data[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, value: BlockId) {
        assert!(Self::is_within_padded_bounds(x, y, z));
        self.set_unpadded((x + 1) as usize, (y + 1) as usize, (z + 1) as usize, value);
    }

    pub fn update(&mut self, x: i32, y: i32, z: i32, value: BlockId) {
        self.set(x, y, z, value);

        if !value.supports_grass()
            && Self::is_within_padded_bounds(x, y + 1, z)
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
