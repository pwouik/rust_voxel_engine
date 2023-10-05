struct Output{
    @builtin(position) vertex_pos: vec4f,
    @location(0) tex_coord: vec2f,
    @location(1) @interpolate(flat) tex_id: u32,
    @location(2) @interpolate(flat) light: u32,
};
struct Face{
    pos_dir:u32,
    tex:u32,
    light:u32,
};

var<push_constant> region: u32;

@group(0) @binding(0) var<uniform> viewproj: mat4x4<f32>;
@group(1) @binding(0) var<storage,read> faces: array<Face>;


fn face_vertex(vertex_id:u32)->vec3f{
    return vec3f(f32(extractBits(0xCC69F0u,vertex_id,1u)),f32(extractBits(0xF0CCCCu,vertex_id,1u)),f32(extractBits(0x69F096u,vertex_id,1u)));
}
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32,@builtin(instance_index) instance_index: u32)-> Output{
    var uv = array<vec2f,4>(
        vec2f(1.0,1.0),
        vec2f(0.0,1.0),
        vec2f(0.0,0.0),
        vec2f(1.0,0.0));
    var indices = array<u32,6>(0u,1u,2u,2u,3u,0u);
    let face_vertex_id:u32 = indices[vertex_index%6u];
    let face_id:u32 = vertex_index/6u;
    let pos_dir:u32 = faces[face_id].pos_dir;
    let pos:vec4f = ((unpack4x8unorm(instance_index) * 255.0) + ((unpack4x8unorm(region) * 255.0) - 128.0)) * 32.0;
    var ret:Output;
    ret.vertex_pos = viewproj * (vec4f(pos.xyz + vec3f(
        f32(pos_dir&0x000000FFu),
        f32(extractBits(pos_dir,8u,8u)),
        f32(extractBits(pos_dir,16u,8u))) + face_vertex(extractBits(pos_dir,24u,8u)*4u+face_vertex_id), 1.0));
    ret.tex_coord = uv[face_vertex_id];
    ret.tex_id = faces[face_id].tex;
    ret.light = faces[face_id].light;
    return ret;
}


@group(2) @binding(0) var textures:texture_2d_array<f32>;
@group(2) @binding(1) var texture_sampler:sampler;

@fragment
fn fs_main(pos_in: Output)->   @location(0) vec4f {
     let lights:vec4f = unpack4x8unorm(pos_in.light);
     return textureSample(textures,texture_sampler,pos_in.tex_coord,i32(pos_in.tex_id))*mix(mix(lights[2],lights[3],pos_in.tex_coord.x),mix(lights[1],lights[0],pos_in.tex_coord.x),pos_in.tex_coord.y);
}