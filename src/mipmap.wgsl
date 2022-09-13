@compute
var<push_constant> mip_level:i32;

@group(0) @binding(0)
var texture_in:texture_storage_2d_array<rgba8unorm,read>;
@group(0) @binding(1)
var texture_out:texture_storage_2d_array<rgba8unorm,write>;

@compute @workgroup_size(4,4,1)
fn main(@builtin(global_invocation_id) global_id_u: vec3<u32>){
    let global_id=vec3<i32>(global_id_u);
    let p1 = textureLoad(texture_in,vec2<i32>(global_id.x*2,global_id.y*2),global_id.z);
    let p2 = textureLoad(texture_in,vec2<i32>(global_id.x*2+1,global_id.y*2),global_id.z);
    let p3 = textureLoad(texture_in,vec2<i32>(global_id.x*2,global_id.y*2+1),global_id.z);
    let p4 = textureLoad(texture_in,vec2<i32>(global_id.x*2+1,global_id.y*2+1),global_id.z);
    textureStore(texture_out,vec2<i32>(global_id.x,global_id.y),global_id.z,(p1+p2+p3+p4)/4.0);
}