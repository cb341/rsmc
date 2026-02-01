use bevy::prelude::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    WaitingForServer,
    LoadingSpawnRegion,
    Chatting,
    Debugging,
    Playing,
}
