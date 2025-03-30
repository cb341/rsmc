// Vertex shader for a Bevy PBR material extension
// The shader preprocessor directives are processed by Bevy before compilation

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings mesh
#import bevy_pbr::mesh_functions

struct MyExtensionMaterial {
    quantize_steps: u32,
};

@group(2) @binding(100)
var<uniform> my_extension: MyExtensionMaterial;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Get model matrix
    let model = mesh_functions::get_world_from_local(instance_index);
    
    // Transform to world space
    let world_position = model * vec4<f32>(position, 1.0);
    
    // Transform to clip space
    out.clip_position = mesh_view_bindings::view.view_proj * world_position;
    
    // Save world position
    out.world_position = world_position;
    
    // Transform normal to world space (simplification)
    out.world_normal = (model * vec4<f32>(normal, 0.0)).xyz;
    
    // Pass UV coordinates
    out.uv = uv;
    
    return out;
}

// Fragment shader not needed as we're extending the StandardMaterial and using its fragment shader
