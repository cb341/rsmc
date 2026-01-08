use std::collections::HashMap;

use bevy::{math::Vec3, prelude::Resource};

use crate::CullType;

use super::BlockId;

pub const CHUNK_SIZE: usize = 30;
pub const PADDED_CHUNK_SIZE: usize = CHUNK_SIZE + 2;

pub const PADDED_CHUNK_USIZE: usize = PADDED_CHUNK_SIZE;
pub const CHUNK_LENGTH: usize = PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE;
pub const TOTAL_BLOCKS_PER_CHUNK: usize = CHUNK_LENGTH;

const REGION_WIDTH: usize = PADDED_CHUNK_SIZE / REGION_COUNT_PER_SIDE_OF_CHUNK;
const REGION_COUNT_PER_SIDE_OF_CHUNK: usize = 4;
const RC: usize = REGION_COUNT_PER_SIDE_OF_CHUNK;
const BLOCK_COUNT_PER_REGION: usize = REGION_WIDTH * REGION_WIDTH * REGION_WIDTH;
const TOTAL_REGIONS_PER_CHUNK: usize = TOTAL_BLOCKS_PER_CHUNK / BLOCK_COUNT_PER_REGION;

#[derive(Debug, Clone, Copy, Default)]
struct SubRegion {
    pub solid_count: u16,
    pub mixed_count: u16,
}

impl SubRegion {
    pub fn is_empty(&self) -> bool {
        self.solid_count + self.mixed_count == 0
    }

    pub fn contains_blocks(&self) -> bool {
        !self.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub sub_regions: [SubRegion; TOTAL_REGIONS_PER_CHUNK],
    pub data: [BlockId; CHUNK_LENGTH],
    pub position: Vec3,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            sub_regions: [SubRegion::default(); TOTAL_REGIONS_PER_CHUNK],
            data: [BlockId::Air; CHUNK_LENGTH],
            position: Vec3::ZERO,
        }
    }
}

impl Chunk {
    fn xyz_from_index(n: usize, index: usize) -> (usize, usize, usize) {
        let x = index % n;
        let y = (index / n) % n;
        let z = index / (n * n);

        (x, y, z)
    }

    pub fn block_iterator(&self) -> impl Iterator<Item = (usize, usize, usize, BlockId)> {
        let mut region_index = 0;
        let mut block_index = 0;

        std::iter::from_fn(move || {
            while region_index < TOTAL_REGIONS_PER_CHUNK {
                let region = self.sub_regions[region_index];
                if region.is_empty() { // TODO: expose this
                    region_index += 1;
                    block_index = 0;
                    continue;
                }

                let rw = REGION_WIDTH;
                let rc = RC;

                let (rx, ry, rz) = Self::xyz_from_index(rc, region_index);
                let (dx, dy, dz) = Self::xyz_from_index(rw, block_index);

                let x = rx * rw + dx;
                let y = ry * rw + dy;
                let z = rz * rw + dz;

                block_index += 1;
                if block_index == BLOCK_COUNT_PER_REGION {
                    region_index += 1;
                    block_index = 0;
                }

                if Self::is_unpadded_pos_at_border(x, y, z) { // TODO: make this optional
                    continue;
                }

                let block = self.data[Self::index(x, y, z)];
                return Some((x, y, z, block));
            }

            None
        })
    }

    fn is_unpadded_pos_at_border(x: usize, y: usize, z: usize) -> bool {
        let min = 0;
        let max = PADDED_CHUNK_SIZE - 1;

        let at_min_border = x == min || y == min || z == min;
        let at_max_border = x == max || y == max || z == max;

        at_min_border || at_max_border
    }
}

impl Chunk {
    // TODO: add optimized iterator over all non empty blocks with triplet
    pub fn with_data(data: [BlockId; CHUNK_LENGTH], position: Vec3) -> Self {
        let mut chunk = Self {
            data,
            position,
            ..Chunk::default()
        };
        chunk.recount_regions();
        chunk
    }

    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Chunk::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sub_regions
            .iter()
            .map(|region| region.solid_count + region.mixed_count)
            .sum::<u16>()
            == 0
    }

    pub fn is_full(&self) -> bool {
        self.sub_regions
            .iter()
            .map(|region| region.solid_count)
            .sum::<u16>()
            == TOTAL_BLOCKS_PER_CHUNK as u16
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
        // println!("Setting ({:?}:{:?}) at ({},{},{})", value, value.cull_type(), x,y,z);

        let value_before = self.data[Self::index(x, y, z)];
        let value_after = value;

        self.data[Self::index(x, y, z)] = value;

        if value_before.cull_type() == value_after.cull_type() {
            return;
        }

        let index = Self::region_index_from_block(x, y, z);
        let region = &mut self.sub_regions[index];

        match value_before.cull_type() {
            CullType::Empty => {}
            CullType::Solid => region.solid_count -= 1,
            CullType::Mixed => region.mixed_count -= 1,
        }
        match value_after.cull_type() {
            CullType::Empty => {}
            CullType::Solid => region.solid_count += 1,
            CullType::Mixed => region.mixed_count += 1,
        }
    }

    pub fn index(x: usize, y: usize, z: usize) -> usize {
        assert!(
            ((x < PADDED_CHUNK_SIZE) && (y < PADDED_CHUNK_SIZE) && (z < PADDED_CHUNK_SIZE)),
            "Index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );

        x + PADDED_CHUNK_USIZE * (y + PADDED_CHUNK_USIZE * z)
    }

    pub fn region_index(x: usize, y: usize, z: usize) -> usize {
        assert!(
            (x < REGION_COUNT_PER_SIDE_OF_CHUNK)
                && (y < REGION_COUNT_PER_SIDE_OF_CHUNK)
                && (z < REGION_COUNT_PER_SIDE_OF_CHUNK),
            "Index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );

        x + REGION_COUNT_PER_SIDE_OF_CHUNK * (y + REGION_COUNT_PER_SIDE_OF_CHUNK * z)
    }

    #[rustfmt::skip]
    pub fn region_index_from_block(x: usize, y: usize, z: usize) -> usize {
        Self::region_index(x / REGION_WIDTH, y / REGION_WIDTH, z / REGION_WIDTH)
    }

    pub fn key_eq_pos(key: [i32; 3], position: Vec3) -> bool {
        position.x as i32 == key[0] && position.y as i32 == key[1] && position.z as i32 == key[2]
    }

    fn fill(&mut self, block_id: BlockId) {
        self.data = [block_id; CHUNK_LENGTH];

        let base_region = match block_id.cull_type() {
            CullType::Solid => SubRegion {
                solid_count: BLOCK_COUNT_PER_REGION as u16,
                mixed_count: 0,
            },
            CullType::Empty => SubRegion::default(),
            CullType::Mixed => SubRegion {
                solid_count: 0,
                mixed_count: BLOCK_COUNT_PER_REGION as u16,
            },
        };
        self.sub_regions = [base_region; TOTAL_REGIONS_PER_CHUNK]
    }

    fn recount_regions(&mut self) {
        for x in 0..PADDED_CHUNK_SIZE {
            for y in 0..PADDED_CHUNK_SIZE {
                for z in 0..PADDED_CHUNK_SIZE {
                    let block_id = self.get_unpadded(x, y, z);
                    let region = &mut self.sub_regions[Self::region_index_from_block(x, y, z)];
                    match block_id.cull_type() {
                        CullType::Empty => {}
                        CullType::Solid => region.solid_count += 1,
                        CullType::Mixed => region.mixed_count += 1,
                    }
                }
            }
        }
    }
}

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
    fn test_chunk_fullness_lifecycle() {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let mut chunk = Chunk::new(position);
        assert!(chunk.is_empty());
        assert!(!chunk.is_full());
        chunk.set(0, 0, 0, BlockId::Air);
        assert!(chunk.is_empty());
        assert!(!chunk.is_full());
        chunk.set(0, 0, 0, BlockId::Stone);
        assert!(!chunk.is_empty());
        assert!(!chunk.is_full());
        chunk.fill(BlockId::Stone);
        assert!(!chunk.is_empty());
        assert!(chunk.is_full());
    }

    #[test]
    fn test_chunk_iterator() {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let mut chunk = Chunk::new(position);
        chunk.block_iterator().for_each(|_| {
            panic!();
        });

        let b = BlockId::Stone;
        chunk.fill(b);
        chunk.set_unpadded(2,1,1, BlockId::Dirt);
        let mut iterator = chunk.block_iterator();
        assert_eq!(iterator.next().unwrap(), (1, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (2, 1, 1, BlockId::Dirt));
        assert_eq!(iterator.next().unwrap(), (3, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (4, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (5, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (6, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (7, 1, 1, b));
        assert_eq!(iterator.next().unwrap(), (1, 2, 1, b));
    }

    #[test]
    fn test_index() {
        assert_eq!(Chunk::region_index(0, 0, 0), 0);
        assert_eq!(Chunk::region_index(1, 0, 0), 1);
        assert_eq!(Chunk::region_index(3, 3, 3), 63);

        assert_eq!(Chunk::region_index_from_block(0, 0, 0), 0);
        assert_eq!(Chunk::region_index_from_block(1, 0, 0), 0);
        assert_eq!(Chunk::region_index_from_block(8, 0, 0), 1);
        assert_eq!(Chunk::region_index_from_block(31, 31, 31), 63);

        assert_eq!(
            Chunk::region_index(1, 2, 3),
            Chunk::region_index_from_block(8, 16, 24)
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
