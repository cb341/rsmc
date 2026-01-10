use crate::prelude::*;

const RAY_DIST: Vec3 = Vec3::new(0.0, 0.0, -20.0);
const HIGHLIGHT_CUBE_ORIGIN: Vec3 = Vec3::new(0.0, 2.0, 0.0);

pub fn setup_highlight_cube_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Cuboid::new(1.01, 1.01, 1.01);

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(Color::srgba(1.0, 1.0, 1.0, 0.5))),
            Transform::from_xyz(0.0, 0.0, -7.0),
        ))
        .insert(player_components::HighlightCube);
}

#[allow(clippy::type_complexity)]
pub fn raycast_system(
    mut raycast: Raycast,
    #[cfg(feature = "raycast_debug")] mut gizmos: Gizmos,
    raycast_origin: Query<&Transform, With<player_components::PlayerCamera>>,
    mut selection_query: Query<
        (&mut Transform, &player_components::HighlightCube),
        (
            Without<player_components::PlayerCamera>,
            Without<player_components::Raycastable>,
        ),
    >,
    raycastable_query: Query<&Transform, With<player_components::Raycastable>>,
    mut block_selection: ResMut<player_resources::BlockSelection>,
) {
    let camera_transform = raycast_origin.single();
    let filter = |entity| raycastable_query.get(entity).is_ok();

    let pos = camera_transform.translation;
    let dir = camera_transform.rotation.mul_vec3(Vec3::Z).normalize();
    let dir = dir * RAY_DIST.z;

    let ray = Ray3d::new(pos, Dir3::new(dir).expect("Ray can be cast"));
    let settings = RaycastSettings {
        filter: &filter,
        ..default()
    };

    #[cfg(feature = "raycast_debug")]
    let intersections = raycast.debug_cast_ray(ray, &settings, &mut gizmos);

    #[cfg(not(feature = "raycast_debug"))]
    let intersections = raycast.cast_ray(ray, &settings);

    let (mut highlight_transform, _) = selection_query.single_mut()?;
    let hover_position = intersections
        .first()
        .map(|(_, intersection)| (intersection.position() - intersection.normal() * 0.5).floor());

    block_selection.position = hover_position;
    block_selection.normal = intersections
        .first()
        .map(|(_, intersection)| intersection.normal());

    if hover_position.is_none() {
        highlight_transform.translation = HIGHLIGHT_CUBE_ORIGIN;
        return;
    }

    highlight_transform.translation = hover_position.unwrap() + 0.5;
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
