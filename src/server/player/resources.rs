use crate::prelude::*;

#[derive(Resource)]
pub struct PlayerStates {
    pub players: HashMap<Username, PlayerState>,
}

impl PlayerStates {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }
}

impl Default for PlayerStates {
    fn default() -> Self {
        Self::new()
    }
}
