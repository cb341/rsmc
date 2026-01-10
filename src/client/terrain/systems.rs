use bevy::tasks::{futures_lite::future, AsyncComputeTaskPool};
use terrain_components::ChunkMesh;
use terrain_resources::{
    ChunkMeshes, FutureChunkMesh, MeshTask, MeshType, MesherTasks, RenderMaterials,
};

use crate::prelude::*;

pub fn prepare_mesher_materials_system(
    mut render_materials: ResMut<RenderMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = obtain_texture_handle(&asset_server);

    let material = create_transparent_material(texture_handle.clone());
    render_materials.transparent_material = Some(materials.add(material));

    let material = create_chunk_material(texture_handle);
    render_materials.chunk_material = Some(materials.add(material));
}

pub fn generate_simple_ground_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mesh = Cuboid::new(64.0, 1.0, 64.0);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgba(1.0, 0.0, 1.0, 1.0))),
        Name::new("Simple Ground Plane"),
    ));
}

pub fn prepare_spawn_area_system(mut client: ResMut<RenetClient>) {
    info!("Sending chunk requests for spawn area");

    let chunks = ChunkManager::instantiate_chunks(IVec3::ZERO, IVec3::ONE);

    let positions: Vec<IVec3> = chunks.into_iter().map(|chunk| chunk.position).collect();
    let message = bincode::serialize(&NetworkingMessage::ChunkBatchRequest(positions));
    info!("requesting world");
    client.send_message(DefaultChannel::ReliableUnordered, message.unwrap());
}

pub fn generate_world_system(
    mut client: ResMut<RenetClient>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let render_distance = IVec3::new(8, 4, 8);

    info!("Sending chunk requests for chunks");

    let chunks = chunk_manager.instantiate_new_chunks(IVec3::ZERO, render_distance);

    let positions: Vec<IVec3> = chunks.into_iter().map(|chunk| chunk.position).collect();

    let batched_positions = positions.chunks(16);
    assert!(batched_positions.len() > 0, "Batched positions is empty");

    batched_positions.enumerate().for_each(|(index, batch)| {
        let request_positions = batch.to_vec();
        info!(
            "Sending chunk batch request for {:?}",
            request_positions.len()
        );
        let message = bincode::serialize(&NetworkingMessage::ChunkBatchRequest(request_positions));
        info!("requesting chunks #{}", index);
        client.send_message(DefaultChannel::ReliableUnordered, message.unwrap());
    });
}

pub fn handle_chunk_mesh_update_events_system(
    chunk_manager: ResMut<ChunkManager>,
    mut chunk_mesh_update_events: EventReader<terrain_events::ChunkMeshUpdateEvent>,
    texture_manager: ResMut<terrain_util::TextureManager>,
    mut tasks: ResMut<MesherTasks>,
) {
    for event in chunk_mesh_update_events.read() {
        info!(
            "Received chunk mesh update event for chunk {:?}",
            event.chunk_position
        );
        let chunk_option = chunk_manager.get_chunk(event.chunk_position);
        match chunk_option {
            Some(chunk) => {
                tasks.task_list.push(FutureChunkMesh {
                    position: chunk.position,
                    meshes_task: create_mesh_task(chunk, &texture_manager),
                });
            }
            None => {
                println!("No chunk found");
            }
        }
    }
}

fn create_mesh_task(chunk: &Chunk, texture_manager: &terrain_util::TextureManager) -> MeshTask {
    let task_pool = AsyncComputeTaskPool::get();
    let chunk = *chunk;
    let texture_manager = texture_manager.clone();
    MeshTask(task_pool.spawn(async move {
        ChunkMeshes {
            cube_mesh: terrain_util::create_cube_mesh_for_chunk(&chunk, &texture_manager),
            cross_mesh: terrain_util::create_cross_mesh_for_chunk(&chunk, &texture_manager),
        }
    }))
}

pub fn handle_chunk_tasks_system(
    mut commands: Commands,
    materials: Res<RenderMaterials>,
    mut tasks: ResMut<MesherTasks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_entities: ResMut<terrain_resources::ChunkEntityMap>,
) {
    let mut completed = 0;
    const MAX_COMPLETIONS: usize = 10; // limit how many meshes we process per cycle
    const KEEP_FOR_NEXT_CYCLE: bool = true;
    const DISCARD: bool = false;

    tasks.task_list.retain_mut(|future_chunk| {
        if completed >= MAX_COMPLETIONS {
            return KEEP_FOR_NEXT_CYCLE;
        }

        let Some(mesh_option) =
            bevy::tasks::block_on(future::poll_once(&mut future_chunk.meshes_task.0))
        else {
            return KEEP_FOR_NEXT_CYCLE;
        };

        completed += 1;
        let pos = future_chunk.position;
        let pos_vec = pos.as_vec3();

        if let Some(entities) = chunk_entities.remove(pos) {
            entities.iter().for_each(|entity| {
                commands.entity(*entity).despawn();
            })
        }

        if let Some(mesh) = mesh_option.cross_mesh {
            let entity = commands
                .spawn(create_chunk_bundle(
                    meshes.add(mesh),
                    pos_vec,
                    MeshType::Transparent,
                    materials.transparent_material.clone().unwrap(),
                ))
                .id();
            chunk_entities.add(pos, entity);
        }

        if let Some(mesh) = mesh_option.cube_mesh {
            let entity = commands
                .spawn(create_chunk_bundle(
                    meshes.add(mesh),
                    pos_vec,
                    MeshType::Solid,
                    materials.chunk_material.clone().unwrap(),
                ))
                .insert(player_components::Raycastable)
                .id();
            chunk_entities.add(pos, entity);
        }

        DISCARD
    });
}

fn create_chunk_bundle(
    mesh_handle: Handle<Mesh>,
    chunk_position: Vec3,
    mesh_type: MeshType,
    material_handle: Handle<StandardMaterial>,
) -> (
    bevy::prelude::Mesh3d,
    bevy::prelude::Transform,
    ChunkMesh,
    bevy::prelude::MeshMaterial3d<StandardMaterial>,
) {
    (
        Mesh3d(mesh_handle),
        Transform::from_xyz(
            chunk_position.x * CHUNK_SIZE as f32,
            chunk_position.y * CHUNK_SIZE as f32,
            chunk_position.z * CHUNK_SIZE as f32,
        ),
        terrain_components::ChunkMesh {
            key: [
                chunk_position.x as i32,
                chunk_position.y as i32,
                chunk_position.z as i32,
            ],
            mesh_type,
        },
        MeshMaterial3d(material_handle),
    )
}

fn create_transparent_material(texture_handle: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        perceptual_roughness: 1.0,
        double_sided: true,
        cull_mode: None,
        reflectance: 0.0,
        unlit: false,
        specular_transmission: 0.0,
        alpha_mode: AlphaMode::Mask(1.0),
        base_color_texture: Some(texture_handle),
        ..default()
    }
}

#[cfg(not(feature = "wireframe"))]
fn create_chunk_material(texture_handle: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        perceptual_roughness: 0.5,
        reflectance: 0.0,
        unlit: false,
        specular_transmission: 0.0,
        base_color_texture: Some(texture_handle),
        ..default()
    }
}

#[cfg(feature = "wireframe")]
fn create_chunk_material(_texture_handle: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    }
}

fn obtain_texture_handle(asset_server: &Res<AssetServer>) -> Handle<Image> {
    asset_server.load("textures/texture_atlas.png")
}

pub fn handle_terrain_regeneration_events_system(
    mut client: ResMut<RenetClient>,
    mut world_regenerate_events: EventReader<terrain_events::WorldRegenerateEvent>,
    chunk_manager: ResMut<ChunkManager>,
) {
    for _ in world_regenerate_events.read() {
        info!("Rerequesting all chunks from server");
        let all_chunk_positions = chunk_manager.get_all_chunk_positions();
        let message =
            bincode::serialize(&NetworkingMessage::ChunkBatchRequest(all_chunk_positions));
        client.send_message(DefaultChannel::ReliableUnordered, message.unwrap());
    }
}
