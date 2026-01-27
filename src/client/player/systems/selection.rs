use crate::prelude::*;

const RAY_DIST: f32 = 20.0;
const HIGHLIGHT_CUBE_ORIGIN: Vec3 = Vec3::MIN;

pub fn setup_highlight_cube_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Cuboid::new(1.01, 1.01, 1.01);

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -7.0),
        ))
        .insert(player_components::HighlightCube);
}

#[allow(clippy::type_complexity)]
pub fn raycast_system(
    mut raycast: MeshRayCast,
    #[cfg(feature = "raycast_debug")] mut gizmos: Gizmos,
    raycast_origin: Query<&Transform, With<player_components::PlayerCamera>>,
    mut selection_query: Query<
        &mut Transform,
        (
            With<player_components::HighlightCube>,
            Without<player_components::PlayerCamera>,
            Without<player_components::Raycastable>,
        ),
    >,
    raycastable_query: Query<Entity, With<player_components::Raycastable>>,
    mut block_selection: ResMut<player_resources::BlockSelection>,
) {
    let camera_transform = raycast_origin
        .single()
        .expect("Camera not present for raycast");

    let pos = camera_transform.translation;
    let dir = camera_transform.rotation.mul_vec3(Vec3::NEG_Z).normalize();

    let ray = Ray3d::new(pos, Dir3::new(dir).expect("Ray can be cast"));

    let binding = |entity| raycastable_query.contains(entity);
    let settings = MeshRayCastSettings::default().with_filter(&binding);

    let hits = raycast.cast_ray(ray, &settings);

    if let Some((_, hit)) = hits.first() {
        if (pos - hit.point).length() < RAY_DIST {
            #[cfg(feature = "raycast_debug")]
            {
                gizmos.line(
                    hit.point + hit.normal,
                    hit.point,
                    Color::srgb(1.0, 0.0, 0.0),
                );
                gizmos.sphere(hit.point, 0.1, Color::srgb(0.0, 0.0, 1.0));
            }

            const CUBE_SIZE: f32 = 1.0;

            let hover_position = (hit.point - hit.normal * (CUBE_SIZE / 2.0)).floor();
            block_selection.position = Some(hover_position);
            block_selection.normal = Some(hit.normal);

            let mut highlight_transform = single_mut!(selection_query);
            highlight_transform.translation = hover_position + (CUBE_SIZE / 2.0);

            return;
        }
    }

    let mut highlight_transform = single_mut!(selection_query);
    highlight_transform.translation = HIGHLIGHT_CUBE_ORIGIN;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_cube_spawned() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, setup_highlight_cube_system)
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<StandardMaterial>::default());

        app.update();

        let highlight_cube_exists = app
            .world_mut()
            .query::<&player_components::HighlightCube>()
            .iter(app.world())
            .count()
            > 0;

        assert!(
            highlight_cube_exists,
            "HighlightCube was not spawned into the world"
        );
    }
}
