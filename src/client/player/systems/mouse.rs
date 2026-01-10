use crate::prelude::*;

pub fn manage_cursor_system(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsController>,
    current_state: Res<State<GameState>>,
) {
    let mut window = window_query.single_mut()?;
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
    mut mouse_events: EventReader<MouseButtonInput>,
    block_selection: Res<player_resources::BlockSelection>,
) {
    if block_selection.normal.is_none() || block_selection.position.is_none() {
        return;
    }

    let position = block_selection.position.unwrap().as_ivec3();
    let normal = block_selection.normal.unwrap().as_ivec3();

    for event in mouse_events.read() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            block_update_events.send(terrain_events::BlockUpdateEvent {
                position,
                block: BlockId::Air,
                from_network: false,
            });
        } else if event.button == MouseButton::Right && event.state.is_pressed() {
            block_update_events.send(terrain_events::BlockUpdateEvent {
                position: position + normal,
                block: BlockId::Dirt,
                from_network: false,
            });
        }
    }
}
