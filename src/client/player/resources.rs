use crate::prelude::*;

#[derive(Resource)]
pub struct BlockSelection {
    pub position: Option<Vec3>,
    pub normal: Option<Vec3>,
}

#[derive(Resource)]
pub struct PlayerSpawned(pub bool);

impl PlayerSpawned {
    pub fn is_spawned(resource: Res<PlayerSpawned>) -> bool {
        resource.0
    }

    pub fn is_not_spawned(resource: Res<PlayerSpawned>) -> bool {
        !resource.0
    }
}

impl Default for BlockSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockSelection {
    pub fn new() -> Self {
        Self {
            position: None,
            normal: None,
        }
    }
}

#[derive(Resource)]
pub struct LastPlayerPosition(pub IVec3);

impl Default for LastPlayerPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl LastPlayerPosition {
    pub fn new() -> Self {
        Self(IVec3::ZERO)
    }

    pub fn chunk_position(&self) -> IVec3 {
        Self::chunk_pos(self.0)
    }

    pub fn has_same_chunk_position_as(&self, other_world_position: IVec3) -> bool {
        Self::chunk_pos(self.0) == Self::chunk_pos(other_world_position)
    }

    fn chunk_pos(world_pos: IVec3) -> IVec3 {
        ChunkManager::world_position_to_chunk_position(world_pos)
    }
}

#[derive(Resource)]
pub struct LocalPlayerSpawnState(pub PlayerState);
