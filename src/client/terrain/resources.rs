use bevy::tasks::Task;

use crate::prelude::*;

#[derive(Resource)]
pub struct SpawnAreaLoaded(pub bool);

impl SpawnAreaLoaded {
    pub fn is_loaded(resource: Res<SpawnAreaLoaded>) -> bool {
        resource.0
    }
}

#[derive(Clone, PartialEq)]
pub enum MeshType {
    Solid,
    Transparent,
}

pub struct ChunkMeshes {
    pub cube_mesh: Option<Mesh>,
    pub cross_mesh: Option<Mesh>,
}

pub struct MeshTask(pub Task<ChunkMeshes>);
pub struct FutureChunkMesh {
    pub position: IVec3,
    pub meshes_task: MeshTask,
}

#[derive(Resource, Default)]
pub struct MesherTasks {
    pub task_list: Vec<FutureChunkMesh>,
}

#[derive(Resource)]
pub struct RenderMaterials {
    pub transparent_material: Option<Handle<StandardMaterial>>,
    pub chunk_material: Option<Handle<StandardMaterial>>,
}

impl Default for RenderMaterials {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderMaterials {
    pub fn new() -> RenderMaterials {
        RenderMaterials {
            transparent_material: None,
            chunk_material: None,
        }
    }
}
