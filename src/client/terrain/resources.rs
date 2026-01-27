use std::{collections::{HashSet, hash_map::ExtractIf}, option::Iter};

use bevy::tasks::Task;

use crate::prelude::*;

#[derive(Resource)]
pub struct SpawnAreaLoaded(pub bool);

impl SpawnAreaLoaded {
    pub fn is_loaded(resource: Res<SpawnAreaLoaded>) -> bool {
        resource.0
    }
}

#[derive(Resource, Default)]
pub struct RequestedChunks {
    pub previous_chunks: HashSet<IVec3>,
}
#[derive(Resource, Default)]
pub struct LastChunkRequestOrigin {
    pub position: IVec3
}

#[derive(Eq, Hash, Clone, PartialEq)]
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

#[derive(Resource, Default)]
pub struct ChunkEntityMap {
    map: HashMap<IVec3, Vec<Entity>>,
}

#[derive(Resource, Default)]
pub struct SpawnArea {
    pub origin: IVec3
}

impl ChunkEntityMap {
    pub fn count(&self) -> usize {
        return self.map.iter().count()
    }

    pub fn add(&mut self, chunk_position: IVec3, entity: Entity) {
        self.map.entry(chunk_position).or_default().push(entity);
    }

    pub fn remove(&mut self, chunk_position: IVec3) -> Option<Vec<Entity>> {
        self.map.remove(&chunk_position)
    }

    pub fn extract_within_distance(&mut self, origin: &IVec3, distance: &IVec3) -> Vec<(IVec3, Vec<Entity>)> {
        let extracted: HashMap<IVec3, Vec<Entity>> = self.map.extract_if(|k, _v| {
            (k.x - origin.x) > distance.x
            || (k.y - origin.y) > distance.y
            || (k.z - origin.z) > distance.z
        }).collect();

        extracted.into_iter().map(|(key, entities)| {
            (key, entities)
        }).collect()
    }
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
