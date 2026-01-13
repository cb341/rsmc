use crate::prelude::*;

#[derive(Message)]
pub struct BlockUpdateEvent {
    pub position: IVec3,
    pub block: BlockId,
}

#[cfg(feature = "generator_visualizer")]
pub use visualizer::*;
#[cfg(feature = "generator_visualizer")]
mod visualizer {
    use super::*;
    use terrain_resources::TextureType;

    #[derive(Message)]
    pub struct RegenerateHeightMapEvent(pub TextureType);

    #[derive(Message)]
    pub struct WorldRegenerateEvent;
}
