use crate::prelude::*;
use bevy::{
    color::palettes::css::RED,
    pbr::MaterialExtension,
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderRef},
};

const SHADER_ASSET_PATH: &str = "shaders/my_material.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct MyExtension {
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for MyExtension {
    fn vertex_shader() -> ShaderRef {
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