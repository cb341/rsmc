use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Indices, PrimitiveTopology};

use crate::prelude::*;

pub fn create_cube_mesh_from_data(geometry_data: GeometryData) -> Option<Mesh> {
    let GeometryData {
        position,
        uv,
        normal,
        indices,
    } = geometry_data;

    if (position.is_empty() || uv.is_empty() || normal.is_empty() || indices.is_empty())
        || (position.len() != uv.len() || uv.len() != normal.len())
    {
        return None;
    }

    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, position)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normal)
        .with_inserted_indices(Indices::U32(indices)),
    )
}

pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

pub struct GeometryData {
    pub position: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub normal: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
