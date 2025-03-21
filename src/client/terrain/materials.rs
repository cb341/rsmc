use crate::prelude::*;
use bevy::{
    color::palettes::css::RED, pbr::MaterialExtension, reflect::Reflect, render::render_resource::{AsBindGroup, ShaderRef}
};

const SHADER_ASSET_PATH: &str = "shaders/my_material.glsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct MyExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for MyExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
        // ShaderRef::Default
        // SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
        // ShaderRef::Default
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn deferred_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(
        pipeline: &MaterialExtensionPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        Ok(())
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
