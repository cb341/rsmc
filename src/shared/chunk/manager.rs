use std::collections::HashMap;

use bevy::{math::IVec3, prelude::Resource};

use crate::*;

#[derive(Resource)]
pub struct ChunkManager {
    pub chunks: HashMap<IVec3, Chunk>,
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

    pub fn instantiate_chunks(position: IVec3, render_distance: IVec3) -> Vec<Chunk> {
        let render_distance_x = render_distance.x;
        let render_distance_y = render_distance.y;
        let render_distance_z = render_distance.z;

        let mut chunks: Vec<Chunk> = Vec::new();

        for x in -render_distance_x..render_distance_x {
            for y in -render_distance_y..render_distance_y {
                for z in -render_distance_z..render_distance_z {
                    let chunk_position = IVec3::new(x + position.x, y + position.y, z + position.z);
                    let chunk = Chunk::new(chunk_position);
                    chunks.push(chunk);
                }
            }
        }

        chunks
    }

    pub fn instantiate_new_chunks(
        &mut self,
        position: IVec3,
        render_distance: IVec3,
    ) -> Vec<Chunk> {
        let chunks = Self::instantiate_chunks(position, render_distance);

        chunks
            .into_iter()
            .filter(|chunk| {
                let chunk_position = chunk.position;
                let chunk = self.get_chunk_mut(&chunk_position);
                chunk.is_none()
            })
            .collect()
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn insert_chunks(&mut self, chunks: Vec<Chunk>) {
        for chunk in chunks {
            self.insert_chunk(chunk);
        }
    }

    pub fn set_chunk(&mut self, position: IVec3, chunk: Chunk) {
        self.chunks.insert(position, chunk);
    }

    pub fn get_chunk(&self, position: IVec3) -> Option<&Chunk> {
        self.chunks.get(&position)
    }

    pub fn get_chunk_mut(&mut self, position: &IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(position)
    }

    pub fn update_block(&mut self, position: IVec3, block: BlockId) -> Vec<IVec3> {
        Self::chunk_positions_containing_world_pos(position)
            .iter()
            .map(|chunk_position| {
                let chunk_option = self.get_chunk_mut(chunk_position);
                match chunk_option {
                    Some(chunk) => {
                        let local_position = IVec3::new(
                            position.x.rem_euclid(CHUNK_SIZE as i32),
                            position.y.rem_euclid(CHUNK_SIZE as i32),
                            position.z.rem_euclid(CHUNK_SIZE as i32),
                        );

                        assert!(local_position.x >= 0 && local_position.x < CHUNK_SIZE as i32);
                        assert!(local_position.y >= 0 && local_position.y < CHUNK_SIZE as i32);
                        assert!(local_position.z >= 0 && local_position.z < CHUNK_SIZE as i32);

                        chunk.update(
                            local_position.x as usize,
                            local_position.y as usize,
                            local_position.z as usize,
                            block,
                        );

                        Some(*chunk_position)
                    }
                    None => {
                        // FIXME: we should do something about updates in unloaded chunks..
                        None
                    }
                }
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect()
    }

    pub fn get_block(&mut self, position: IVec3) -> Option<BlockId> {
        match self.single_chunk_matching_world_position(position) {
            Some(chunk) => {
                let chunk_position = IVec3::new(
                    chunk.position[0] * CHUNK_SIZE as i32,
                    chunk.position[1] * CHUNK_SIZE as i32,
                    chunk.position[2] * CHUNK_SIZE as i32,
                );
                let local_position = position - chunk_position;
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

    fn chunk_positions_containing_world_pos(position: IVec3) -> Vec<IVec3> {
        fn axis_chunks(world: i32) -> Vec<i32> {
            let size = CHUNK_SIZE as i32;
            let base = world.div_euclid(size);

            let candidates = [base - 1, base, base + 1];

            candidates
                .into_iter()
                .filter(|&chunk| {
                    let start = chunk * size - 1;
                    let end = chunk * size + size;
                    world >= start && world <= end
                })
                .collect()
        }

        let xs = axis_chunks(position.x);
        let ys = axis_chunks(position.y);
        let zs = axis_chunks(position.z);

        let mut out = Vec::new();

        for x in xs {
            for y in &ys {
                for z in &zs {
                    out.push(IVec3::new(x, *y, *z));
                }
            }
        }

        out
    }

    fn single_chunk_matching_world_position(&mut self, position: IVec3) -> Option<&mut Chunk> {
        let chunk_position = IVec3 {
            x: position.x.div_euclid(CHUNK_SIZE as i32),
            y: position.y.div_euclid(CHUNK_SIZE as i32),
            z: position.z.div_euclid(CHUNK_SIZE as i32),
        };
        self.get_chunk_mut(&chunk_position)
    }

    pub fn get_all_chunk_positions(&self) -> Vec<IVec3> {
        self.chunks
            .keys()
            .map(|key| IVec3::new(key[0], key[1], key[2]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // For performance reasons chunk copy one block on each side form their neighbor.
    // When updating a block, multiple chunks may be involved.
    // With chunk width = 30:
    //
    // Example chunk x = 0:
    //  -1    0    1
    // [-1][0,29][30]
    //
    //   -2     -1    0
    // [-31][-30, -1][0]
    //
    // The position 0 is included in the padding of chunk -1, as well as in the main area of chuk 0
    #[test]
    fn test_chunk_positions_containing_world_pos() {
        assert_eq!(
            ChunkManager::chunk_positions_containing_world_pos(IVec3::ZERO),
            vec![
                IVec3::new(-1, -1, -1),
                IVec3::new(-1, -1, 0),
                IVec3::new(-1, 0, -1),
                IVec3::new(-1, 0, 0),
                IVec3::new(0, -1, -1),
                IVec3::new(0, -1, 0),
                IVec3::new(0, 0, -1),
                IVec3::new(0, 0, 0)
            ]
        );

        assert_eq!(
            ChunkManager::chunk_positions_containing_world_pos(IVec3::ONE),
            vec![IVec3::new(0, 0, 0),]
        );

        assert_eq!(
            ChunkManager::chunk_positions_containing_world_pos(IVec3::new(0, 1, 1)),
            vec![IVec3::new(-1, 0, 0), IVec3::new(0, 0, 0),]
        );

        assert_eq!(
            ChunkManager::chunk_positions_containing_world_pos(IVec3::new(CHUNK_SIZE as i32, 1, 1)),
            vec![IVec3::new(0, 0, 0), IVec3::new(1, 0, 0),]
        );
    }

    #[test]
    fn test_chunk_manager_new() {
        let chunk_manager = ChunkManager::new();
        assert!(chunk_manager.chunks.is_empty());
    }

    #[test]
    fn test_instantiate_chunks() {
        let position = IVec3::new(0, 0, 0);

        let width = 2;
        let height = 3;
        let depth = 4;

        let render_distance = IVec3::new(width, height, depth);

        let chunks = ChunkManager::instantiate_chunks(position, render_distance);
        assert_eq!(chunks.len(), (2 * width * 2 * height * 2 * depth) as usize,);
    }

    #[test]
    fn test_insert_chunks() {
        let mut chunk_manager = ChunkManager::new();
        let position = IVec3::new(0, 0, 0);
        let render_distance = 2;
        let chunks = ChunkManager::instantiate_chunks(
            position,
            IVec3::new(render_distance, render_distance, render_distance),
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
        let position = IVec3::new(0, 0, 0);
        let chunk = Chunk::new(position);

        chunk_manager.set_chunk(position, chunk);
        let retrieved_chunk = chunk_manager.get_chunk_mut(&position).unwrap();
        assert_eq!(retrieved_chunk.position, chunk.position);
    }

    #[test]
    fn test_set_and_get_block() {
        let mut chunk_manager = ChunkManager::new();
        let position = IVec3::new(0, 0, 0);
        let chunk = Chunk::new(position);

        chunk_manager.set_chunk(position, chunk);
        let block_position = IVec3::new(1, 1, 1);
        let block_id = BlockId::Stone;

        chunk_manager.update_block(block_position, block_id);
        let retrieved_block = chunk_manager.get_block(block_position).unwrap();
        assert_eq!(retrieved_block, block_id);
    }

    #[test]
    fn test_get_all_chunk_positions() {
        let mut chunk_manager = ChunkManager::new();
        chunk_manager.set_chunk(IVec3::new(0, 0, 0), Chunk::default());
        chunk_manager.set_chunk(IVec3::new(2, 0, 0), Chunk::default());
        chunk_manager.set_chunk(IVec3::new(1, 0, 3), Chunk::default());

        let retrieved_chunk_positions = chunk_manager.get_all_chunk_positions();
        assert_eq!(retrieved_chunk_positions.len(), 3);
    }

    #[test]
    #[rustfmt::skip]
    fn test_tallgrass_update() {
        let mut chunk_manager = ChunkManager::new();
        let chunk_position = IVec3::new(0, 0, 0);
        let chunk = Chunk::new(chunk_position);
        chunk_manager.set_chunk(chunk_position, chunk);

        let grass_position = IVec3::new(0, 0, 0);
        let tallgrass_position = IVec3::new(0, 1, 0);

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
