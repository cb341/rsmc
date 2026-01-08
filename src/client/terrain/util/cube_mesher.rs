use terrain_util::{
    GeometryData, TextureManager, Vertex,
    client_block::{MeshRepresentation, block_properties},
    create_cube_mesh_from_data,
};

use crate::prelude::*;

pub fn create_cube_geometry_data(
    x: f32,
    y: f32,
    z: f32,
    faces: u8,
    block_id: BlockId,
    texture_manager: &TextureManager,
) -> GeometryData {
    let mut position = Vec::new();
    let mut uv = Vec::new();
    let mut normal = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0;

    CUBE_FACES.iter().enumerate().for_each(|(i, face)| {
        if faces & (1 << i) == 0 {
            return;
        }

        let face_vertices = face_vertices(*face);
        for vertex in face_vertices.iter() {
            position.push([
                vertex.position[0] * 0.5 + x + 0.5,
                vertex.position[1] * 0.5 + y + 0.5,
                vertex.position[2] * 0.5 + z + 0.5,
            ]);

            let block_uvs = Block::get_block_face_uvs(block_id, *face, texture_manager).unwrap();
            uv.push([
                block_uvs[0] + vertex.uv[0] * 0.25,
                block_uvs[1] + (1.0 - vertex.uv[1]) * 0.25,
            ]);
            normal.push(vertex.normal);
        }

        let offsets = [0, 1, 2, 2, 1, 3];
        offsets.iter().for_each(|offset| {
            indices.push(index_offset + offset);
        });
        index_offset += 4;
    });

    GeometryData {
        position,
        uv,
        normal,
        indices,
    }
}

pub fn create_cube_mesh_for_chunk(chunk: &Chunk, texture_manager: &TextureManager) -> Option<Mesh> {
    let mut geometry_data = GeometryData {
        position: Vec::new(),
        uv: Vec::new(),
        normal: Vec::new(),
        indices: Vec::new(),
    };
    let instant = Instant::now();

    for x in 1..CHUNK_SIZE + 1 {
        for y in 1..CHUNK_SIZE + 1 {
            for z in 1..CHUNK_SIZE + 1 {
                let block_id = chunk.get_unpadded(x, y, z);

                match block_properties(block_id).mesh_representation {
                    MeshRepresentation::Cube(_) => {}
                    _ => continue,
                }

                fn update_mask(
                    chunk: &Chunk,
                    mask: &mut u8,
                    value: u8,
                    x: usize,
                    y: usize,
                    z: usize,
                ) {
                    match block_properties(chunk.get_unpadded(x, y, z)).mesh_representation {
                        MeshRepresentation::Cube(_) => {}
                        _ => *mask |= value,
                    }
                }

                let mut mask = 0b000000;

                update_mask(chunk, &mut mask, 0b000001, x, y + 1, z);
                update_mask(chunk, &mut mask, 0b000010, x, y - 1, z);

                update_mask(chunk, &mut mask, 0b000100, x + 1, y, z);
                update_mask(chunk, &mut mask, 0b001000, x - 1, y, z);

                update_mask(chunk, &mut mask, 0b010000, x, y, z - 1);
                update_mask(chunk, &mut mask, 0b100000, x, y, z + 1);

                let cube_data = create_cube_geometry_data(
                    (x - 1) as f32,
                    (y - 1) as f32,
                    (z - 1) as f32,
                    mask,
                    block_id,
                    texture_manager,
                );

                geometry_data.indices.extend(
                    cube_data
                        .indices
                        .iter()
                        .map(|i| i + geometry_data.position.len() as u32),
                );
                geometry_data.position.extend(cube_data.position);
                geometry_data.uv.extend(cube_data.uv);
                geometry_data.normal.extend(cube_data.normal);
            }
        }
    }

    let old = instant.elapsed();

    let mut geometry_data = GeometryData {
        position: Vec::new(),
        uv: Vec::new(),
        normal: Vec::new(),
        indices: Vec::new(),
    };

    let instant = Instant::now();

    chunk.block_iterator().for_each(|(x, y, z, block_id)| {
        match block_properties(block_id).mesh_representation {
            MeshRepresentation::Cube(_) => {}
            _ => return,
        }

        fn update_mask(chunk: &Chunk, mask: &mut u8, value: u8, x: usize, y: usize, z: usize) {
            match block_properties(chunk.get_unpadded(x, y, z)).mesh_representation {
                MeshRepresentation::Cube(_) => {}
                _ => *mask |= value,
            }
        }

        let mut mask = 0b000000;

        update_mask(chunk, &mut mask, 0b000001, x, y + 1, z);
        update_mask(chunk, &mut mask, 0b000010, x, y - 1, z);

        update_mask(chunk, &mut mask, 0b000100, x + 1, y, z);
        update_mask(chunk, &mut mask, 0b001000, x - 1, y, z);

        update_mask(chunk, &mut mask, 0b010000, x, y, z - 1);
        update_mask(chunk, &mut mask, 0b100000, x, y, z + 1);

        let cube_data = create_cube_geometry_data(
            (x - 1) as f32,
            (y - 1) as f32,
            (z - 1) as f32,
            mask,
            block_id,
            texture_manager,
        );

        geometry_data.indices.extend(
            cube_data
                .indices
                .iter()
                .map(|i| i + geometry_data.position.len() as u32),
        );
        geometry_data.position.extend(cube_data.position);
        geometry_data.uv.extend(cube_data.uv);
        geometry_data.normal.extend(cube_data.normal);
    });

    let new = instant.elapsed();

    println!("[{:?}, {:?}],", old.as_nanos(), new.as_nanos());

    create_cube_mesh_from_data(geometry_data)
}

#[derive(Debug, Clone, Copy)]
pub enum CubeFace {
    Top,
    Bottom,
    Right,
    Left,
    Back,
    Forward,
}

const CUBE_FACES: [CubeFace; 6] = [
    CubeFace::Top,
    CubeFace::Bottom,
    CubeFace::Right,
    CubeFace::Left,
    CubeFace::Back,
    CubeFace::Forward,
];

#[rustfmt::skip]
fn face_vertices(face_index: CubeFace) -> [Vertex; 4] {
    match face_index {
        CubeFace::Left => [
            Vertex{ position: [-1.0, -1.0, -1.0], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex{ position: [-1.0, -1.0, 1.0], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0] },
            Vertex{ position: [-1.0, 1.0, -1.0], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0] },
            Vertex{ position: [-1.0, 1.0, 1.0], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        ],
        CubeFace::Right => [
            Vertex{ position: [1.0, -1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex{ position: [1.0, -1.0, -1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
            Vertex{ position: [1.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
            Vertex{ position: [1.0, 1.0, -1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        ],
        CubeFace::Bottom => [
            Vertex{ position: [1.0, -1.0, 1.0], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0] },
            Vertex{ position: [-1.0, -1.0, 1.0], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0] },
            Vertex{ position: [1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0] },
            Vertex{ position: [-1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0] },
        ],
        CubeFace::Top => [
            Vertex{ position: [1.0, 1.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
            Vertex{ position: [-1.0, 1.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
            Vertex{ position: [1.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
            Vertex{ position: [-1.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
        ],
        CubeFace::Back => [
            Vertex{ position: [1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
            Vertex{ position: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
            Vertex{ position: [1.0, 1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
            Vertex{ position: [-1.0, 1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
        ],
        CubeFace::Forward => [
            Vertex{ position: [-1.0, -1.0, 1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] },
            Vertex{ position: [1.0, -1.0, 1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
            Vertex{ position: [-1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
            Vertex{ position: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0] }
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrain_util::TextureManager;

    #[test]
    fn test_create_cube_mesh_from_data() {
        let geometry_data = GeometryData {
            position: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [
                0.0, 1.0, 0.0,
            ]],
            uv: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            normal: vec![[0.0, 0.0, 1.0]; 4],
            indices: vec![0, 1, 2, 2, 3, 0],
        };

        let mesh = create_cube_mesh_from_data(geometry_data);
        assert!(mesh.is_some());
    }

    #[test]
    fn test_create_cube_geometry_data() {
        let texture_manager = TextureManager::new();
        let geometry_data =
            create_cube_geometry_data(0.0, 0.0, 0.0, 0b111111, BlockId::Stone, &texture_manager);

        assert_eq!(geometry_data.position.len(), 6 * 4);
        assert_eq!(geometry_data.uv.len(), 6 * 4);
        assert_eq!(geometry_data.normal.len(), 6 * 4);
        assert_eq!(geometry_data.indices.len(), 6 * 6);
    }
}
