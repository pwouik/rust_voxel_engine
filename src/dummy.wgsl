struct Output{
    @builtin(position) frame_pos: vec4<f32>,
    @location(1) vertex_pos: vec4<f32>,
};
struct Block{
    viewproj :mat4x4<f32>,
};
struct Block2{
    boxes :array<u32>,
};

@group(0) @binding(0)
var<uniform> viewproj: Block;


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32)-> Output{
    var ret:Output;
    ret.vertex_pos = vec4<f32>(f32(vertex_index/2u)*2.0 - 1.0,f32(vertex_index%2u)*2.0 - 1.0,0.1,1.0);
    ret.frame_pos = ret.vertex_pos;
    return ret;
};

fn imod(a:i32,b:i32)->i32{
    return (a%b+b)%b;
}

@group(1) @binding(0)
var depth:texture_2d<f32>;
@group(1) @binding(1)
var depth_sampler:sampler;

@fragment
fn fs_main(pos_in: Output)-> @location(0) vec4<f32>{
    let tex_pos = pos_in.vertex_pos / pos_in.vertex_pos.w;
    var c=textureSample(depth, depth_sampler, vec2<f32>(0.9,0.9)).r;
    return vec4<f32>((tex_pos.y+1.0)/2.0,c,c,1.0);
}