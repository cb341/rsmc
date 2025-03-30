use crate::prelude::*;
use bevy::{
    color::palettes::css::RED,
    pbr::MaterialExtension,
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderRef},
};

const SHADER_ASSET_PATH: &str = "shaders/my_material.wgsl";

// Material extension that allows quantizing geometry positions
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct MyExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for MyExtension {
    fn vertex_shader() -> ShaderRef {
        // Only use our shader for function overrides, not for the full vertex shader
        SHADER_ASSET_PATH.into()
    }
}

pub fn create_base_material(
    texture_handle: Handle<Image>,
) -> ExtendedMaterial<StandardMaterial, MyExtension> {
    ExtendedMaterial {
        base: StandardMaterial {
            opaque_render_method: OpaqueRendererMethod::Auto,
            perceptual_roughness: 0.5,
            reflectance: 0.0,
            unlit: false,
            specular_transmission: 0.0,
            base_color_texture: Some(texture_handle),
            ..default()
        },
        extension: MyExtension { quantize_steps: 12 },
    }
}

/// Register the material in Bevy's asset system
pub fn setup_custom_material(
    mut commands: Commands, 
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Load a texture
    let texture_handle = asset_server.load("textures/terrain_texture.png");
    
    // Create a material using our extension
    let material = create_base_material(texture_handle);
    let material_handle = materials.add(material);
    
    // Create a mesh
    let mesh_handle = meshes.add(Cuboid::default());
    
    // Example: Create a cube with the custom material
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Add this function to your App setup
pub fn register_material_extension(app: &mut App) {
    app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyExtension>>::default())
       .add_systems(Startup, setup_custom_material);
}
