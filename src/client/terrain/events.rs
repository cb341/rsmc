use crate::prelude::*;

#[derive(Event)]
pub struct ChunkMeshUpdateEvent {
    pub position: IVec3,
}

#[derive(Event)]
pub struct BlockUpdateEvent {
    pub position: IVec3,
    pub block: BlockId,
    pub from_network: bool,
}

#[derive(Event)]
pub struct WorldRegenerateEvent;
