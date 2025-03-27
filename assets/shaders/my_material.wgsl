struct VertexInput {
    @location(0) position: vec4f,
    @location(1) texcoords: vec2f,
}

//struct VertexOutput {
//    @builtin(position) position: vec4f,
//    @location(1) texcoords: vec2f,
//}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec3<f32>,
}


struct FragmentInput {
    @location(1) texcoords: vec2f,
}

struct FragmentOutput {
    @location(0) color: vec4f,
}

@group(0) @binding(0) var<uniform> matrix: mat4x4f;
@group(0) @binding(1) var baseTexture: texture_2d<f32>;
@group(0) @binding(2) var baseSampler: sampler;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = matrix * input.position;
    // output.texcoords = input.texcoords;

    return output;
}

@fragment
fn fragment(input: FragmentInput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = textureLoad(baseTexture, input.texcoords, baseSampler);
    return output;
}
