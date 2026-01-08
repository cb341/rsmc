use terrain_util::{
    GeometryData, TextureManager, Vertex,
    client_block::{MeshRepresentation, block_properties},
    create_cube_mesh_from_data,
};

use crate::prelude::*;

pub fn create_cross_mesh_for_chunk(
    chunk: &Chunk,
    texture_manager: &TextureManager,
) -> Option<Mesh> {
    let geometry_data = create_cross_geometry_for_chunk(chunk, texture_manager);

    create_cube_mesh_from_data(geometry_data)
}

fn create_cross_geometry_for_chunk(
    chunk: &Chunk,
    texture_manager: &TextureManager,
) -> GeometryData {
    let mut position = vec![];
    let mut uv = vec![];
    let mut normal = vec![];
    let mut indices = vec![];

    let mut index_offset = 0;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_id = chunk.get(x, y, z);
                let pos = Vec3::new(x as f32, y as f32, z as f32);
                let mesh_repr = block_properties(block_id).mesh_representation;

                if let MeshRepresentation::Cross(textures) = mesh_repr {
                    CrossFace::values().iter().for_each(|cross_face| {
                        let face_verticies = cross_face_vertices(*cross_face);

                        let face_uv = texture_manager
                            .get_texture_uv(textures[0])
                            .expect("Texture is not present in manager");

                        for vertex in face_verticies {
                            position.push([
                                pos.x + vertex.position[0] * 0.5 + 0.5,
                                pos.y + vertex.position[1] * 0.5 + 0.5,
                                pos.z + vertex.position[2] * 0.5 + 0.5,
                            ]);

                            uv.push([
                                face_uv[0] + vertex.uv[0] * 0.25,
                                face_uv[1] + vertex.uv[1] * 0.25,
                            ]);
                            normal.push(vertex.normal);
                        }

                        let offsets = [0, 1, 3, 1, 2, 3];
                        offsets.iter().for_each(|offset| {
                            indices.push(index_offset + offset);
                        });

                        index_offset += 4;
                    });
                }
            }
        }
    }

    GeometryData {
        position,
        uv,
        normal,
        indices,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CrossFace {
    Face1,
    Face2,
}

impl CrossFace {
    pub fn values() -> [CrossFace; 2] {
        [CrossFace::Face1, CrossFace::Face2]
    }
}

#[rustfmt::skip]
fn cross_face_vertices(face: CrossFace) -> [Vertex; 4] {
    match face {
        CrossFace::Face1 => [
            Vertex{ position: [-1.0,  1.0, -1.0], normal: [FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [0.0, 0.0] },
            Vertex{ position: [ 1.0,  1.0,  1.0], normal: [FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [1.0, 0.0] },
            Vertex{ position: [ 1.0, -1.0,  1.0], normal: [FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [1.0, 1.0] },
            Vertex{ position: [-1.0, -1.0, -1.0], normal: [FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [0.0, 1.0] },
        ],
        CrossFace::Face2 => [
            Vertex{ position: [-1.0,  1.0,  1.0], normal: [-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [0.0, 0.0] },
            Vertex{ position: [ 1.0,  1.0, -1.0], normal: [-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [1.0, 0.0] },
            Vertex{ position: [ 1.0, -1.0, -1.0], normal: [-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [1.0, 1.0] },
            Vertex{ position: [-1.0, -1.0,  1.0], normal: [-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2], uv: [0.0, 1.0] },
        ],
    }
}
