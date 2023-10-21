struct Output{
    @builtin(position) frame_pos: vec4f,
    @location(0) @interpolate(flat) chunk_pos: vec3i,
    @location(1) vertex_pos: vec4f,
};

@group(0) @binding(0) var<uniform> viewproj: mat4x4<f32>;

@group(2) @binding(0) var<storage,read> boxes: array<u32>;


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32)-> Output{
    var indices = array<u32,6>(0u,1u,2u,2u,3u,0u);
    var face_vertex_id:u32 = indices[vertex_index%6u];
    let pos:u32 = boxes[vertex_index/18u];
    var ret:Output;
    ret.chunk_pos.x = i32(pos & 255u) - 128;
    ret.chunk_pos.y = i32(extractBits( pos, 8u, 8u)) - 128;
    ret.chunk_pos.z = i32(extractBits( pos, 16u, 8u)) - 128;
    let dir:u32 = (vertex_index%18u)/6u;
    var face:u32;
    if(dir==0u)
    {
        if(ret.chunk_pos.x>0){
            face=0u;
        }
        else{face=1u;}
    }
    else if(dir==1u){
        if(ret.chunk_pos.z>0){
            face=2u;
        }
        else{face=3u;}
    }
    else if(ret.chunk_pos.y>0){
        face=4u;
    }
    else{face=5u;}
    face_vertex_id = face * 4u + face_vertex_id;
    ret.vertex_pos = viewproj * (vec4f(f32(ret.chunk_pos.x + i32(extractBits(0xCC69F0u,face_vertex_id,1u))),
                               f32(ret.chunk_pos.y + i32(extractBits(0xF0CCCCu,face_vertex_id,1u))),
                               f32(ret.chunk_pos.z + i32(extractBits(0x69F096u,face_vertex_id,1u))), 1.0) * vec4f(32.0,32.0,32.0,1.0));
    if(ret.chunk_pos.x == 0 && ret.chunk_pos.y == 0 && ret.chunk_pos.z == 0){ret.vertex_pos = vec4f(f32(face_vertex_id/2u)*0.1,f32(face_vertex_id%2u)*0.1,0.1,1.0);}
    ret.frame_pos = ret.vertex_pos;

    return ret;
};

struct Context{
    player_pos: vec3i,
    height: i32,
    width: i32,
};
@group(0) @binding(1) var<uniform> context: Context;

@group(1) @binding(0) var depth:texture_2d<f32>;
@group(1) @binding(1) var depth_sampler:sampler;

@group(2) @binding(1) var<storage,read_write> visibility: array<u32>;

fn imod(a:i32,b:i32)->i32{
    return (a%b+b)%b;
}

@fragment
fn fs_main(pos_in: Output){
    let tex_pos = pos_in.vertex_pos / pos_in.vertex_pos.w * vec4(1.0,-1.0,1.0,1.0);
    //let d = textureSampleCompare(depth, depth_sampler, tex_pos.xy*0.5 + 0.5).r;//textureSampleCompare(depth,depth_sampler,tex_pos.xy*0.5 + 0.5,0.0);
    var c=0.5f;
    var d=textureSample(depth, depth_sampler, tex_pos.xy*0.5 + 0.5).r - pos_in.frame_pos.z;
    if(textureSample(depth, depth_sampler, tex_pos.xy*0.5 + 0.5).r >= pos_in.frame_pos.z){
        let pos:vec3i = context.player_pos + pos_in.chunk_pos;
        let id:u32 = u32((pos.x & 15) + 16*(pos.y & 7) + 16*8*(pos.z & 15))*6u;
        if(pos_in.chunk_pos.x>=0){
            visibility[id]=1u;
        }
        if(pos_in.chunk_pos.x<=0){
            visibility[id+1u]=1u;
        }
        if(pos_in.chunk_pos.z>=0 ){
            visibility[id+2u]=1u;
        }
        if(pos_in.chunk_pos.z<=0){
            visibility[id+3u]=1u;
        }
        if(pos_in.chunk_pos.y>=0){
            visibility[id+4u]=1u;
        }
        if(pos_in.chunk_pos.y<=0){
            visibility[id+5u]=1u;
        }
    }
}