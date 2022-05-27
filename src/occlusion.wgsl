struct Output{
    [[builtin(position)]] vertex_pos: vec4<f32>;
    [[location(0), interpolate(flat)]] chunk_pos: vec3<i32>;
};
struct Block{
    viewproj :mat4x4<f32>;
};
struct Block2{
    boxes :array<u32>;
};
[[group(0), binding(0)]]
var<uniform> viewproj: Block;
[[group(1), binding(0)]]
var<storage,read> boxes: Block2;


[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32, )-> Output{
    var indices = array<u32,6>(0u,1u,2u,2u,3u,0u);
    var face_vertex_id:u32 = indices[vertex_index%6u];
    let pos:u32 = boxes.boxes[vertex_index/18u];
    var ret:Output;
    ret.chunk_pos.x = i32(pos & 255u) - 128;
    ret.chunk_pos.y = i32(extractBits( pos, 8u, 8u)) - 128;
    ret.chunk_pos.z = i32(extractBits( pos, 16u, 8u)) - 128;
    let dir:u32 = (vertex_index%18u)/6u;
    var face:u32;
    if(dir==0u)
    {
        if(ret.chunk_pos.x<0){
            face=0u;
        }
        else{face=1u;}
    }
    else if(dir==1u){
        if(ret.chunk_pos.z<0){
            face=2u;
        }
        else{face=3u;}
    }
    else if(ret.chunk_pos.y<0){
        face=4u;
    }
    else{face=5u;}
    face_vertex_id = face * 6u + face_vertex_id;
    ret.vertex_pos = vec4<f32>(f32(extractBits(0xCC69F0u,face_vertex_id,1u)),f32(extractBits(0xCC69F0u,face_vertex_id,1u)),f32(extractBits(0xCC69F0u,face_vertex_id,1u)), 1.0) * viewproj.viewproj;
    return ret;
};

struct Context{
    player_pos: vec3<i32>;
    height: i32;
    width: i32;
};

[[group(0), binding(1)]]
var<uniform> context: Context;

struct Block3{
    buf :array<u32>;
};
[[group(1), binding(1)]]
var<storage,read_write> visibility: Block3;

[[stage(fragment)]]
fn fs_main(pos_in: Output){
    var pos:vec3<i32> = context.player_pos + pos_in.chunk_pos;
    let id:u32 = u32(pos.x % context.width) + u32(context.width*(pos.y%context.height)) + u32(context.width*context.height*(pos.z%context.width));
    if(pos_in.chunk_pos.x<=0){
        visibility.buf[id]=1u;
    }
    if(pos_in.chunk_pos.x>=0){
        visibility.buf[id+1u]=1u;
    }
    if(pos_in.chunk_pos.y<=0){
        visibility.buf[id+2u]=1u;
    }
    if(pos_in.chunk_pos.y>=0){
        visibility.buf[id+3u]=1u;
    }
    if(pos_in.chunk_pos.z<=0){
        visibility.buf[id+4u]=1u;
    }
    if(pos_in.chunk_pos.z>=0){
        visibility.buf[id+5u]=1u;
    }
}