#define_import_path bevy_pbr::my_material

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::forward_io::VertexOutput
#import bevy_render::maths::affine3_to_square

struct MyExtensionMaterial {
    quantize_steps: u32,
};

@group(2) @binding(100)
var<uniform> my_extension: MyExtensionMaterial;

fn quantize_position(position: vec3<f32>, steps: u32) -> vec3<f32> {
    if (steps > 0u) {
        let step_size = 1.0 / f32(steps);
        return floor(position / step_size) * step_size;
    }
    return position;
}

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    let model_affine = mesh_bindings::mesh[instance_index].world_from_local;
    let model = affine3_to_square(model_affine);
    
    let world_position = model * vec4<f32>(position, 1.0);
    
    var quantized_position = world_position;
    let quantized_xyz = quantize_position(world_position.xyz, my_extension.quantize_steps);
    quantized_position.x = quantized_xyz.x;
    quantized_position.y = quantized_xyz.y;
    quantized_position.z = quantized_xyz.z;
    
    out.position = mesh_view_bindings::view.clip_from_world * quantized_position;
    
    out.world_position = world_position;
    
    out.world_normal = (model * vec4<f32>(normal, 0.0)).xyz; 
    
    out.uv = uv;
    
    return out;
}