struct Output{
    [[builtin(position)]] vertex_pos: vec4<f32>;
    [[location(0)]] chunk_pos: vec3<i32>;
    [[location(1), interpolate(flat)]] chunk_pos: vec3<i32>;
    [[location(3), interpolate(flat)]] chunk_pos: vec3<i32>;
};
struct Face{
    pos_dir:u32;
    tex:u32;
    light:u32;
};
struct Block{
    buf:array<Face>;
};

[[group(0), binding(0)]]
var<uniform> viewproj: Block;
[[group(1), binding(0)]]
var<storage,read> faces: Block;

let uv = array<vec2<f32>,4>(
vec2<f32>(1.0,1.0),
vec2<f32>(0.0,1.0),
vec2<f32>(0.0,0.0),
vec2<f32>(1.0,0.0));

fn face_vertex(vertex_id:u32)->vec3<f32>{
    return vec3<f32>(extractBits(0xCC69F0u,vertex_id,1),extractBits(0xCC69F0u,vertex_id,1),extractBits(0xCC69F0u,vertex_id,1));
}
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32, )-> Output{
    var indices = array<u32,6>(0u,1u,2u,2u,3u,0u);
    let face_vertex_id:u32 = indices[vertex_index%6u];
    let face_id:u32 = vertex_index/6u;
    let pos_dir:u32 = faces.buf[face_id+vertex_index].pos_dir;
    let pos:vec4<f32> = (unpackUnorm4x8(gl_BaseInstance) * 255 - 128)* 32;

    gl_Position = view_proj * (vec4(pos.xyz + vec3(pos_dir&0x000000FF,bitfieldExtract(pos_dir,8,8),bitfieldExtract(pos_dir,16,8)) + face_vertex(bitfieldExtract(pos_dir,24,8)*4+face_vertex_id), 1.0));
    tex_coord = uv[face_vertex_id];
    tex_id = faces_data[face_id].texture;
    light = faces_data[face_id].light;
}