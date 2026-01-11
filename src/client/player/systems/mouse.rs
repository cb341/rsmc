use bevy::window::PrimaryWindow;

use crate::prelude::*;

pub fn manage_cursor_system(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut controller_query: Query<&mut FpsController>,
    current_state: Res<State<GameState>>,
) {
    let mut window = single_mut!(window_query);
    if btn.just_pressed(MouseButton::Left) && *current_state.get() != GameState::Debugging {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        for mut controller in &mut controller_query {
            controller.enable_input = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        for mut controller in &mut controller_query {
            controller.enable_input = false;
        }
    }
}

pub fn handle_mouse_events_system(
    mut block_update_events: EventWriter<terrain_events::BlockUpdateEvent>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    block_selection: Res<player_resources::BlockSelection>,
) {
    if block_selection.normal.is_none() || block_selection.position.is_none() {
        return;
    }

    let position = block_selection.position.unwrap().as_ivec3();
    let normal = block_selection.normal.unwrap().as_ivec3();

    if mouse_buttons.just_pressed(MouseButton::Left) {
        block_update_events.write(terrain_events::BlockUpdateEvent {
            position,
            block: BlockId::Air,
            from_network: false,
        });
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        block_update_events.write(terrain_events::BlockUpdateEvent {
            position: position + normal,
            block: BlockId::Dirt,
            from_network: false,
        });
    }
}
