use terrain_resources::{Generator, NoiseFunctionParams, TerrainGeneratorParams};

use crate::{prelude::*, terrain::resources::Noise};

macro_rules! for_each_chunk_coordinate {
    ($chunk:expr, $body:expr) => {
        for x in 0..CHUNK_SIZE + 2 {
            for y in 0..CHUNK_SIZE + 2 {
                for z in 0..CHUNK_SIZE + 2 {
                    #[cfg(feature = "skip_chunk_padding")]
                    if x == 0
                        || x == CHUNK_SIZE + 1
                        || y == 0
                        || y == CHUNK_SIZE + 1
                        || z == 0
                        || z == CHUNK_SIZE + 1
                    {
                        continue;
                    }

                    let chunk_origin = $chunk.position * CHUNK_SIZE as i32;
                    let local_position = IVec3::new(x as i32, y as i32, z as i32);
                    let world_position = chunk_origin + local_position;

                    $body(x, y, z, world_position);
                }
            }
        }
    };
}

impl Generator {
    pub fn new(seed: u32) -> Generator {
        Generator {
            noise: Noise::new(seed),
            params: TerrainGeneratorParams::default(),
        }
    }

    pub fn generate_chunk(&self, chunk: &mut Chunk) {
        for_each_chunk_coordinate!(chunk, |x, y, z, world_position| {
            let block = self.generate_block(world_position);
            chunk.set_unpadded(x, y, z, block);
        });

        for_each_chunk_coordinate!(chunk, |x, y, z, _| {
            let pos = IVec3::new(x as i32, y as i32, z as i32);

            self.decorate_block(chunk, pos);
        });

        for _ in 0..self.params.tree.spawn_attempts_per_chunk {
            self.attempt_spawn_tree(chunk);
        }
    }

    fn attempt_spawn_tree(&self, chunk: &mut Chunk) {
        let proposal = Self::propose_tree_blocks(self);

        struct Bounds {
            min: IVec3,
            max: IVec3,
        }

        let proposal_bounds = proposal.iter().fold(
            Bounds {
                min: IVec3::ZERO,
                max: IVec3::ZERO,
            },
            |bounds, (relative_pos, _block_id)| Bounds {
                min: IVec3 {
                    x: bounds.min.x.min(relative_pos.x),
                    y: bounds.min.y.min(relative_pos.y),
                    z: bounds.min.z.min(relative_pos.z),
                },
                max: IVec3 {
                    x: bounds.max.x.max(relative_pos.x),
                    y: bounds.max.y.max(relative_pos.y),
                    z: bounds.max.z.max(relative_pos.z),
                },
            },
        );

        let sapling_x: i32 = rand::random_range(
            proposal_bounds.min.x.abs()..(CHUNK_SIZE as i32 - proposal_bounds.max.x),
        );
        let sapling_y: i32 = rand::random_range(
            proposal_bounds.min.y.abs()..(CHUNK_SIZE as i32 - proposal_bounds.max.y),
        );
        let sapling_z: i32 = rand::random_range(
            proposal_bounds.min.z.abs()..(CHUNK_SIZE as i32 - proposal_bounds.max.z),
        );

        if chunk.get(sapling_x, sapling_y, sapling_z) != BlockId::Grass {
            return;
        }

        let proposal_valid = proposal.iter().all(|(relative_pos, _block)| {
            let IVec3 { x, y, z } = relative_pos;
            Chunk::is_within_padded_bounds(
                sapling_x as i32 + { *x },
                sapling_y as i32 + { *y },
                sapling_z as i32 + { *z },
            ) && chunk.get(
                sapling_x as i32 + { *x },
                sapling_y as i32 + { *y },
                sapling_z as i32 + { *z },
            ) == BlockId::Air
        });

        if !proposal_valid {
            return;
        }

        proposal.iter().for_each(|(relative_pos, block_id)| {
            let IVec3 { x, y, z } = relative_pos;
            chunk.set(
                sapling_x as i32 + { *x },
                sapling_y as i32 + { *y },
                sapling_z as i32 + { *z },
                *block_id,
            );
        });
    }

    fn propose_tree_blocks(&self) -> Vec<(IVec3, BlockId)> {
        let mut blocks = Vec::new();

        let min_tree_stump_height = self.params.tree.min_stump_height;
        let max_tree_stump_height = self.params.tree.max_stump_height;

        let tree_stump_height =
            rand::random_range(min_tree_stump_height..max_tree_stump_height) as i32;

        let bush_radius: i32 =
            rand::random_range(self.params.tree.min_bush_radius..self.params.tree.max_bush_radius)
                as i32;

        for dx in -bush_radius..bush_radius {
            for dz in -bush_radius..bush_radius {
                for dy in -bush_radius..bush_radius {
                    let distance_from_center = dx * dx + dy * dy + dz * dz;
                    if distance_from_center < bush_radius * bush_radius {
                        blocks.push((
                            IVec3 {
                                x: dx,
                                y: tree_stump_height + dy,
                                z: dz,
                            },
                            BlockId::OakLeaves,
                        ));
                    }
                }
            }
        }

        for dy in 1..tree_stump_height {
            blocks.push((IVec3 { x: 0, y: dy, z: 0 }, BlockId::OakLog));
        }

        blocks
    }

    fn decorate_block(&self, chunk: &mut Chunk, position: IVec3) {
        let x = position.x as usize;
        let y = position.y as usize;
        let z = position.z as usize;

        let block = chunk.get_unpadded(x, y, z);

        if block == BlockId::Air {
            if y > 0
                && Chunk::valid_unpadded(x, y - 1, z)
                && chunk.get_unpadded(x, y - 1, z) == BlockId::Grass
            {
                let random_number = rand::random_range(0..=self.params.grass.frequency);
                if random_number == 0 {
                    chunk.set_unpadded(x, y, z, BlockId::Tallgrass);
                }
            }
            return;
        }

        let mut depth_below_nearest_air = 0;
        let depth_check = 3;

        for delta_height in 0..depth_check {
            if !Chunk::valid_unpadded(x, y + delta_height, z) {
                break;
            }

            let block = chunk.get_unpadded(x, y + delta_height, z);

            if block == BlockId::Air {
                break;
            }

            depth_below_nearest_air += 1;
        }

        let block = match depth_below_nearest_air {
            0_i32..=1_i32 => BlockId::Grass,
            2..3 => BlockId::Dirt,
            _ => BlockId::Stone,
        };

        chunk.set_unpadded(x, y, z, block);
    }

    fn generate_block(&self, position: IVec3) -> BlockId {
        if self.is_inside_cave(position.as_vec3()) {
            return BlockId::Air;
        }

        if (position.y as f64) < self.determine_terrain_height(position.as_vec3()) {
            return BlockId::Stone;
        }

        if self.determine_terrain_density(position.as_vec3()) > 0.0 {
            return BlockId::Stone;
        }

        BlockId::Air
    }

    fn is_inside_cave(&self, position: Vec3) -> bool {
        let density = self.sample_3d(position, &self.params.cave.noise);

        let upper_bound = self.params.cave.base_value - self.params.cave.threshold;
        let lower_bound = self.params.cave.base_value + self.params.cave.threshold;

        lower_bound <= density && density >= upper_bound
    }

    fn determine_terrain_height(&self, position: Vec3) -> f64 {
        let noise_value = self
            .sample_2d(
                Vec2 {
                    x: position.x,
                    y: position.z,
                },
                &self.params.height.noise,
            )
            .abs();

        self.spline_lerp(noise_value)
    }

    fn determine_terrain_density(&self, position: Vec3) -> f64 {
        let density = self.sample_3d(position, &self.params.density.noise);
        let density_falloff = (position.y as f64 + self.params.density.height_offset)
            * self.params.density.squash_factor;

        density - density_falloff
    }

    pub fn normalized_spline_terrain_sample(&self, position: Vec2) -> f64 {
        let noise_value = self.sample_2d(position, &self.params.height.noise);

        let min_height = self.params.height.splines[0].y as f64;
        let max_height = self.params.height.splines[self.params.height.splines.len() - 1].y as f64;

        let splined_value = self.spline_lerp(noise_value);

        (splined_value - min_height) / (max_height - min_height)
    }

    fn spline_lerp(&self, x: f64) -> f64 {
        let x: f32 = x as f32;

        assert!(self.params.height.splines.len() >= 2);

        let min_x = self.params.height.splines[0].x;
        let max_x = self.params.height.splines[self.params.height.splines.len() - 1].x;

        assert!(min_x == -1.0);
        assert!(max_x == 1.0);

        for i in 0..self.params.height.splines.len() - 1 {
            let current = self.params.height.splines[i];
            let next = self.params.height.splines[i + 1];

            if x >= current.x && x <= next.x {
                return self.lerp(current, x, next);
            }
        }

        panic!("Could not find matching spline points for x value {}", x);
    }

    fn lerp(&self, point0: Vec2, x: f32, point1: Vec2) -> f64 {
        ((point0.y * (point1.x - x) + point1.y * (x - point0.x)) / (point1.x - point0.x)) as f64
    }

    pub fn sample_2d(&self, position: Vec2, params: &NoiseFunctionParams) -> f64 {
        let mut sample = 0.0;
        let mut frequency = params.frequency;
        let mut weight = 1.0;
        let mut weight_sum = 0.0;

        for _ in 0..params.octaves {
            let new_sample = self
                .noise
                .get_2d([position.x as f64 * frequency, position.y as f64 * frequency]);

            frequency *= params.lacuranity;
            sample += new_sample * weight;
            weight_sum += weight;
            weight *= params.persistence;
        }

        sample / weight_sum
    }

    pub fn sample_3d(&self, position: Vec3, params: &NoiseFunctionParams) -> f64 {
        let mut sample = 0.0;
        let mut frequency = params.frequency;
        let mut weight = 1.0;
        let mut weight_sum = 0.0;

        for _ in 0..params.octaves {
            let new_sample = self.noise.get([
                position.x as f64 * frequency,
                position.y as f64 * frequency,
                position.z as f64 * frequency,
            ]);

            frequency *= params.lacuranity;
            sample += new_sample * weight;
            weight_sum += weight;
            weight *= params.persistence;
        }

        sample / weight_sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrain_resources::Generator;

    #[test]
    fn test_generate_chunk() {
        let generator = Generator::default();
        let mut chunk = Chunk::new(IVec3::ZERO);

        generator.generate_chunk(&mut chunk);

        assert_ne!(chunk.get(0, 0, 0), BlockId::Air);
    }
}
