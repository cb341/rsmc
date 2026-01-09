use crate::prelude::*;

#[derive(Event)]
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

    #[derive(Event)]
    pub struct RegenerateHeightMapEvent(pub TextureType);

    #[derive(Event)]
    pub struct WorldRegenerateEvent;
}
