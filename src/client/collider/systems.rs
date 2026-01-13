use terrain_util::client_block::block_properties;

use crate::prelude::*;

static COLLIDER_GRID_SIZE: u32 = 4;
static COLLIDER_RESTING_POSITION: Vec3 = Vec3::ZERO;
static COLLIDER_CUBOID_WIDTH: f32 = 1.0;

pub fn setup_coliders_system(mut commands: Commands) {
    let collider_range = 0..COLLIDER_GRID_SIZE;

    commands.spawn((
        Collider::cuboid(256.0, 1.0, 256.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    for x in collider_range.clone() {
        for y in collider_range.clone() {
            for z in collider_range.clone() {
                commands
                    .spawn((
                        Collider::cuboid(
                            COLLIDER_CUBOID_WIDTH / 2.0,
                            COLLIDER_CUBOID_WIDTH / 2.0,
                            COLLIDER_CUBOID_WIDTH / 2.0,
                        ),
                        Transform::from_xyz(x as f32, y as f32, z as f32),
                    ))
                    .insert(collider_components::BlockCollider {
                        relative_position: Vec3 {
                            x: x as f32 - (COLLIDER_GRID_SIZE as f32) / 2.0,
                            y: y as f32 - (COLLIDER_GRID_SIZE as f32) / 2.0,
                            z: z as f32 - (COLLIDER_GRID_SIZE as f32) / 2.0,
                        },
                    });
            }
        }
    }
}

pub fn handle_collider_update_events_system(
    mut collider_grid_events: MessageReader<collider_events::ColliderUpdateEvent>,
    mut query: Query<(&mut Transform, &collider_components::BlockCollider)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for event in collider_grid_events.read() {
        let event_position = Vec3::new(
            event.grid_center_position[0],
            event.grid_center_position[1],
            event.grid_center_position[2],
        )
        .floor();
        for (mut transform, collider) in query.iter_mut() {
            let relative_position = collider.relative_position;
            let collider_position = (event_position + relative_position).floor();

            let block = chunk_manager.get_block(collider_position.as_ivec3());

            match block {
                Some(block) => {
                    if block_properties(block).has_collider {
                        transform.translation = collider_position + COLLIDER_CUBOID_WIDTH / 2.0;
                    } else {
                        transform.translation = COLLIDER_RESTING_POSITION;
                    }
                }
                None => {
                    transform.translation = COLLIDER_RESTING_POSITION;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use collider_events::ColliderUpdateEvent;

    use super::*;
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn test_setup_coliders_system() {
        let mut app = setup_app();
        app.add_systems(Startup, setup_coliders_system);

        app.update();

        let mut colliders_query = app
            .world_mut()
            .query::<&collider_components::BlockCollider>();
        let colliders_count = colliders_query.iter(app.world_mut()).count();

        assert_eq!(colliders_count, 4 * 4 * 4);
    }

    #[test]
    fn test_handle_collider_update_events_system() {
        let mut app = App::new();

        app.add_message::<collider_events::ColliderUpdateEvent>();
        app.add_systems(Update, handle_collider_update_events_system);
        app.insert_resource(ChunkManager::new());

        app.world_mut().spawn((
            Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..Default::default()
            },
            collider_components::BlockCollider {
                relative_position: Vec3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
            },
        ));

        let block = BlockId::Dirt;
        let mut resource = app.world_mut().get_resource_mut::<ChunkManager>().unwrap();
        let chunks = ChunkManager::instantiate_chunks(IVec3::ZERO, IVec3::ONE);
        resource.insert_chunks(chunks);
        resource.update_block(IVec3 { x: 6, y: 7, z: 8 }, block);

        app.world_mut().write_message(ColliderUpdateEvent {
            grid_center_position: [5.0, 5.0, 5.0],
        });

        app.update();

        let mut collider_query = app
            .world_mut()
            .query::<(&Transform, &collider_components::BlockCollider)>();
        let world_mut = app.world_mut();
        let (collider_transform, _) = collider_query.single(world_mut).unwrap();
        assert_eq!(
            Vec3 {
                x: 6.5,
                y: 7.5,
                z: 8.5
            },
            collider_transform.translation
        );
    }
}
