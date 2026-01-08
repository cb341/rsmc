use bevy::math::Vec3;

use crate::{BlockId, CHUNK_LENGTH};

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub data: [BlockId; CHUNK_LENGTH],
    pub position: Vec3,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}
