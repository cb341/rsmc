use crate::prelude::*;

#[derive(Message)]
pub struct ColliderUpdateEvent {
    pub grid_center_position: [f32; 3],
}
