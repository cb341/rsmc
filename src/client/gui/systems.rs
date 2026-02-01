use bevy::window::CursorOptions;
use bevy_flair::style::components::NodeStyleSheet;

use crate::prelude::*;

pub fn setup_gui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            ..default()
        },
    ));
    commands.spawn((
        Node::default(),
        Name::new("menu_title_wrapper"),
        NodeStyleSheet::new(asset_server.load("gui.css")),
    ));
}

pub fn handle_debug_state_transition_system(
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if key_input.just_pressed(KeyCode::Tab) {
        match *current_state.get() {
            GameState::LoadingSpawnRegion => {}
            GameState::WaitingForServer => {}
            GameState::Playing => next_state.set(GameState::Debugging),
            GameState::Chatting => next_state.set(GameState::Debugging),
            GameState::Debugging => next_state.set(GameState::Playing),
        }
    }
}

pub fn handle_enter_debug_state_system(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}
