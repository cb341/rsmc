use crate::prelude::*;

pub fn setup_player_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Player cam?"),
        Camera3d::default(),
        #[cfg(not(feature = "ortho_camera"))]
        Projection::Perspective(PerspectiveProjection {
            fov: TAU / 5.0,
            ..default()
        }),
        #[cfg(feature = "ortho_camera")]
        Projection::Orthographic(OrthographicProjection {
            scale: 0.125,
            near: 0.0001,
            far: 1000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            area: Rect::new(-1.0, -1.0, 1.0, 1.0),
            ..OrthographicProjection::default_3d()
        }),
        RenderPlayer {
            logical_entity: Entity::from_raw_u32(0).unwrap(),
        },
        player_components::PlayerCamera,
    ));
}

pub fn setup_controller_on_area_ready_system(
    mut commands: Commands,
    mut player_spawned: ResMut<player_resources::PlayerSpawned>,
    mut render_player: Query<&mut RenderPlayer>,
    spawn_state: Res<player_resources::LocalPlayerSpawnState>,
) {
    let spawn_position = spawn_state.0.position;
    info!("Setting up controller at {:?}", spawn_position);

    let logical_entity = commands
        .spawn((
            Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            #[cfg(feature = "lock_player")]
            RigidBody::Fixed,
            #[cfg(not(feature = "lock_player"))]
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            Ccd { enabled: true },
            Transform::from_translation(spawn_position),
            LogicalPlayer,
            #[cfg(not(feature = "lock_player"))]
            {
                let (yaw, pitch, _roll) = spawn_state.0.rotation.to_euler(EulerRot::YXZ);
                FpsControllerInput {
                    pitch,
                    yaw,
                    ..default()
                }
            },
            #[cfg(feature = "lock_player")]
            FpsControllerInput {
                pitch: 0.0,
                yaw: 0.0,
                ..default()
            },
            FpsController {
                upright_height: 1.8,
                height: 1.8,
                crouch_height: 1.2,
                air_acceleration: 80.0,
                radius: 0.75,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.2,
        })
        .insert(player_components::Player)
        .id();

    let mut player = render_player
        .single_mut()
        .expect("Failed to query render_player");
    player.logical_entity = logical_entity;

    player_spawned.0 = true;
}

pub fn handle_controller_movement_system(
    query: Query<(Entity, &FpsControllerInput, &Transform)>,
    mut last_position: ResMut<player_resources::LastPlayerPosition>,
    mut collider_events: MessageWriter<collider_events::ColliderUpdateEvent>,
    mut terrain_events: MessageWriter<terrain_events::RerequestChunks>,
    mut cleanup_events: MessageWriter<terrain_events::CleanupChunksAroundOrigin>,
) {
    for (_entity, _input, transform) in &mut query.iter() {
        let controller_position: IVec3 = transform.translation.as_ivec3();

        if last_position.0 != controller_position {
            collider_events.write(collider_events::ColliderUpdateEvent {
                grid_center_position: [
                    // TODO: refactor colliders to use integers over floats
                    controller_position.x as f32,
                    controller_position.y as f32,
                    controller_position.z as f32,
                ],
            });

            if !last_position.has_same_chunk_position_as(controller_position) {
                info!("Player moved out of chunk, rerequesting chunks for: {controller_position}");
                terrain_events.write(terrain_events::RerequestChunks {
                    center_chunk_position: ChunkManager::world_position_to_chunk_position(
                        controller_position,
                    ),
                });
                cleanup_events.write(terrain_events::CleanupChunksAroundOrigin {
                    center_chunk_position: ChunkManager::world_position_to_chunk_position(
                        controller_position,
                    ),
                });
            }
        }
        last_position.0 = controller_position;
    }
}

pub fn activate_fps_controller_system(mut controller_query: Query<&mut FpsController>) {
    for mut controller in &mut controller_query.iter_mut() {
        controller.enable_input = true;
    }
}

pub fn lock_cursor_system(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

pub fn deactivate_fps_controller_system(mut controller_query: Query<&mut FpsController>) {
    for mut controller in &mut controller_query.iter_mut() {
        controller.enable_input = false;
    }
}
