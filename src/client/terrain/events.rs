use crate::prelude::*;

#[derive(Message)]
pub struct ChunkMeshUpdateEvent {
    pub chunk_position: IVec3,
}

#[derive(Message)]
pub struct BlockUpdateEvent {
    pub position: IVec3,
    pub block: BlockId,
    pub from_network: bool,
}

#[derive(Message)]
pub struct WorldRegenerateEvent;
