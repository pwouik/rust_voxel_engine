// Vertex shader

[[block]]
struct Uniforms {
    view_proj: mat4x4<f32>;
    faces:array<vec4<f32>,24>;
    uv:array<vec4<f32>,4>;
};
[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;
struct InstanceInput {
    [[location(1)]] pos: u32;
    [[location(2)]] texture: u32;
};
struct VertexInput {
    [[location(0)]] vertex_id: u32;
};
struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]

fn main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let texture_pos = vec2<f32>(f32(instance.texture & u32(15))/16.0 ,f32(instance.texture>>u32(4))/16.0);
    var out: VertexOutput;
    out.tex_coords = vec2<f32>(uniforms.uv[vertex.vertex_id].x,uniforms.uv[vertex.vertex_id].y) + texture_pos;
    out.clip_position = uniforms.view_proj * (vec4<f32>(f32(instance.pos&u32(255)),
                                                        f32((instance.pos&u32(65280))>>u32(8)),
                                                        f32((instance.pos&u32(16711680))>>u32(16)),
                                                         1.0)+
                                                         (uniforms.faces[((instance.pos&u32(4278190080))>>u32(24))*u32(4)+vertex.vertex_id]));
    return out;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}