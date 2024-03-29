struct Context{
    player_pos: vec3i,
    height: i32,
    width: i32,
};

struct Command{
    vertex_count:u32,
    instance_count:u32,
    base_vertex:u32,
    base_instance:u32,
};
@group(0) @binding(1) var<uniform> context: Context;

@group(1) @binding(0) var<storage, read_write> commands:array<Command>;
@group(1) @binding(1) var<storage, read> index:array<u32>;
@group(1) @binding(2) var<storage, read_write> visibility:array<u32>;
@group(1) @binding(3) var<storage, read_write> count:atomic<u32>;

@compute @workgroup_size(96,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let x = global_id.x/6u;
    let face = global_id.x % 6u;
    let pos = (x + 16u * global_id.y + 16u * 8u * global_id.z);
    let visibility_pos = pos * 6u + global_id.x % 6u;
    let offset_pos = visibility_pos + pos;//location for a specific face;
    if(visibility[visibility_pos]==1u){
        let id = atomicAdd(&count,1u);
        commands[id].vertex_count = u32(index[offset_pos+1u] - index[offset_pos])*6u;
        commands[id].instance_count = 1u;
        commands[id].base_vertex = index[offset_pos]*6u;
        commands[id].base_instance = insertBits(insertBits(x,global_id.y,8u,8u),global_id.z,16u,8u);
    }
    visibility[visibility_pos]=0u;
}
