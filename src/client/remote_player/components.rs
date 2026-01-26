use crate::prelude::*;

#[derive(Component)]
pub struct RemotePlayer {
    pub username: Username,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RemotePlayerGizmos;
