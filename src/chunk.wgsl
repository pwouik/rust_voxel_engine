struct Output{
    @builtin(position) vertex_pos: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) @interpolate(flat) tex_id: u32,
    @location(2) @interpolate(flat) light: u32,
};
struct Face{
    pos_dir:u32,
    tex:u32,
    light:u32,
};
struct Block{
    viewproj :mat4x4<f32>,
};
struct Block1{
    buf:array<Face>,
};
@group(0) @binding(0)
var<uniform> viewproj: Block;
@group(1) @binding(0)
var<storage,read> faces: Block1;

fn face_vertex(vertex_id:u32)->vec3<f32>{
    return vec3<f32>(f32(extractBits(0xCC69F0u,vertex_id,1u)),f32(extractBits(0xF0CCCCu,vertex_id,1u)),f32(extractBits(0x69F096u,vertex_id,1u)));
}
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32,@builtin(instance_index) instance_index: u32)-> Output{
    var uv = array<vec2<f32>,4>(
    vec2<f32>(1.0,1.0),
    vec2<f32>(0.0,1.0),
    vec2<f32>(0.0,0.0),
    vec2<f32>(1.0,0.0));
    var indices = array<u32,6>(0u,1u,2u,2u,3u,0u);
    let face_vertex_id:u32 = indices[vertex_index%6u];
    let face_id:u32 = vertex_index/6u;
    let pos_dir:u32 = faces.buf[face_id].pos_dir;
    let pos:vec4<f32> = ((unpack4x8unorm(instance_index) * 255.0) - 128.0)* 32.0;
    var ret:Output;
    ret.vertex_pos = viewproj.viewproj * (vec4<f32>(pos.xyz + vec3<f32>(
        f32(pos_dir&0x000000FFu),
        f32(extractBits(pos_dir,8u,8u)),
        f32(extractBits(pos_dir,16u,8u))) + face_vertex(extractBits(pos_dir,24u,8u)*4u+face_vertex_id), 1.0));
    ret.tex_coord = uv[face_vertex_id];
    ret.tex_id = faces.buf[face_id].tex;
    ret.light = faces.buf[face_id].light;
    return ret;
}


@group(2) @binding(0)
var textures:texture_2d_array<f32>;
@group(2) @binding(1)
var texture_sampler:sampler;

@fragment
fn fs_main(pos_in: Output)->   @location(0) vec4<f32> {
     let lights:vec4<f32> = unpack4x8unorm(pos_in.light);
     return textureSample(textures,texture_sampler,pos_in.tex_coord,i32(pos_in.tex_id))*mix(mix(lights[2],lights[3],pos_in.tex_coord.x),mix(lights[1],lights[0],pos_in.tex_coord.x),pos_in.tex_coord.y);
}