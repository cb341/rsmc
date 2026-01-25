use crate::prelude::*;
use bevy_mod_billboard::prelude::*;

pub fn spawn_remote_player_system(
    mut commands: Commands,
    mut spawn_events: MessageReader<remote_player_events::RemotePlayerSpawnedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let terminus_handle = asset_server.load("fonts/Terminus500.ttf");

    for event in spawn_events.read() {
        let username = event.username.clone();

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..default()
        });

        commands
            .spawn((
                Node::default(),
                bevy::prelude::Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                MeshMaterial3d(material),
                remote_player_components::RemotePlayer {
                    username: username.clone(),
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node::default(),
                        BillboardText::default(),
                        TextLayout::new_with_justify(Justify::Center),
                        Transform::from_scale(Vec3::splat(0.0085)),
                    ))
                    .with_child((
                        Node::default(),
                        TextSpan::new(format!("{username}\n\n\n")),
                        TextFont::from(terminus_handle.clone()).with_font_size(60.0),
                        TextColor::from(Color::WHITE),
                    ));
            });
    }
}

pub fn despawn_remote_player_system(
    mut commands: Commands,
    mut despawn_events: MessageReader<remote_player_events::RemotePlayerDespawnedEvent>,
    query: Query<(Entity, &remote_player_components::RemotePlayer)>,
) {
    for event in despawn_events.read() {
        for (entity, remote_player) in query.iter() {
            if remote_player.username == event.username {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn update_remote_player_system(
    mut sync_events: MessageReader<remote_player_events::RemotePlayerSyncEvent>,
    mut spawn_events: MessageWriter<remote_player_events::RemotePlayerSpawnedEvent>,
    mut query: Query<(&remote_player_components::RemotePlayer, &mut Transform)>,
) {
    let latest_event = sync_events.read().last();

    if let Some(event) = latest_event {
        for (username, player_state) in event.players.iter() {
            let mut player_exists = false;
            for (remote_player, mut transform) in query.iter_mut() {
                if remote_player.username == *username {
                    player_exists = true;
                    transform.translation = player_state.position + Vec3::new(0.0, 1.55, 0.0);
                    transform.rotation = player_state.rotation;
                }
            }

            if !player_exists {
                spawn_events.write(remote_player_events::RemotePlayerSpawnedEvent {
                    username: username.clone(),
                    position: player_state.position,
                });
            }
        }
    }
}

pub fn draw_gizmos(
    mut player_gizmos: Gizmos<remote_player_components::RemotePlayerGizmos>,
    query: Query<(&remote_player_components::RemotePlayer, &Transform)>,
) {
    for (_, transform) in query.iter() {
        player_gizmos.ray(
            transform.translation,
            transform.rotation * Vec3::new(0.0, 0.0, -1.0),
            Color::srgb(0.8, 0.7, 0.6),
        );
    }
}
