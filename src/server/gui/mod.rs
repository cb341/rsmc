use bevy::ecs::system::Commands;
use bevy_camera::Camera2d;

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}
