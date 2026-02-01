use crate::prelude::*;

#[derive(Message)]
pub struct ChunkMeshUpdateEvent {
    pub chunk_position: IVec3,
}

#[derive(Message)]
pub struct RerequestChunks {
    pub center_chunk_position: IVec3,
}

#[derive(Message)]
pub struct RequestChunkBatch {
    pub positions: Vec<IVec3>,
}

#[derive(Message)]
pub struct CleanupChunksAroundOrigin {
    pub center_chunk_position: IVec3,
}

#[derive(Message)]
pub struct BlockUpdateEvent {
    pub position: IVec3,
    pub block: BlockId,
    pub from_network: bool,
}

#[derive(Message)]
pub struct WorldRegenerateEvent;
