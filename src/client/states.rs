use bevy::prelude::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    WaitingForServer,
    Chatting,
    Debugging,
    Playing,
}
