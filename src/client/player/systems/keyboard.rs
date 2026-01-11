use crate::prelude::*;

pub fn handle_keyboard_events_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    camera_query: Query<&Transform, With<player_components::HighlightCube>>,
    mut collider_events: EventWriter<collider_events::ColliderUpdateEvent>,
    controller_query: Query<&FpsController>,
) {
    for controller in &controller_query {
        for event in keyboard_events.read() {
            if !controller.enable_input {
                continue;
            }

            if event.state.is_pressed() && event.key_code == bevy::input::keyboard::KeyCode::KeyC {
                let controller_transform = single!(camera_query);
                println!("Handling event: {:?}", controller_transform.translation);
                collider_events.send(collider_events::ColliderUpdateEvent {
                    grid_center_position: controller_transform.translation.into(),
                });
            }
        }
    }
}
