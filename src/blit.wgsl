struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) tex_coords: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var vertex_out: VertexOutput;
    let x = i32(vertex_index) / 2;
    let y = i32(vertex_index) & 1;
    let tc = vec2f(
        f32(x)*2.0,
        f32(y)*2.0
    );
    vertex_out.position = vec4f(
        tc.x * 2.0 - 1.0,
        1.0 - tc.y * 2.0,
        0.0, 1.0
    );
    vertex_out.tex_coords = tc;
    return vertex_out;
}

@group(0) @binding(0)
var r_color: texture_2d<f32>;
@group(0) @binding(1)
var r_sampler: sampler;

@fragment
fn fs_main(vertex_in: VertexOutput) -> @location(0) vec4f {
    return textureSample(r_color, r_sampler, vertex_in.tex_coords);
}