// Vertex shader

struct ViewUniform {
    view_proj: mat4x4<f32>,
    view_rotation: mat4x4<f32>,
    inv_screen_size: vec2<f32>,
    resolution: f32,
}

@group(0) @binding(1)
var<uniform> transform: ViewUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) norm: vec2<f32>,
    @location(3) norm_limit: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    @location(10) bundle_anchor: vec3<f32>,
    @location(11) bundle_opacity: f32,
) -> VertexOutput {
    var out: VertexOutput;
    var color = model.color;
    color[3] = model.color[3] * bundle_opacity;
    out.color = color;

    var norm_length = sqrt(model.norm[0] * model.norm[0] + model.norm[1] * model.norm[1]) * transform.resolution;

    let norm = model.norm * transform.resolution;
    let position = model.position + bundle_anchor;
    let vertex_position = transform.view_proj * vec4<f32>(position.xy + norm, position[2], 1.0);

    out.clip_position = vertex_position;

    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
