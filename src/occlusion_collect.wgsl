struct Context{
    player_pos: vec3<i32>,
    height: i32,
    width: i32,
};

struct Counter {
    c: atomic<u32>,
};
struct Block1{
    mask:array<u32>,
};
struct Command{
    vertex_count:u32,
    instance_count:u32,
    base_vertex:u32,
    base_instance:u32,
};
struct Block2{
    data:array<Command>,
};
struct Block3{
    data:array<u32>,
};

@group(0) @binding(1)
var<uniform> context: Context;

@group(1) @binding(0)
var<storage, read_write> count: Counter;

@group(1) @binding(1)
var<storage, read> visibility:Block1;//circular bounded

@group(1) @binding(2)
var<storage, read_write> commands:Block2;

@group(1) @binding(3)
var<storage, read> chunk_table:Block3;

fn imod(a:i32,b:i32)->i32{
    return (a%b+b)%b;
}

@compute @workgroup_size(128,6,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if(i32(global_id.x) >= context.width*context.height*context.width)
    {
        //return;
    }
    if(global_id.x==0u && global_id.y==0u){
        count.c=0u;
    }
    storageBarrier();
    if(i32(global_id.x) < context.width*context.height*context.width && visibility.mask[global_id.x*6u + global_id.y]==1u){
        let id = atomicAdd(&count.c,1u);
        var pos = i32(global_id.x);
        let x = imod(pos - context.player_pos.x + (context.width - 1)/2, context.width) - (context.width - 1)/2 + 128;
        pos = pos/context.width;
        let y = imod(pos - context.player_pos.y + (context.height - 1)/2, context.height) - (context.height - 1)/2 + 128;
        pos = pos/context.height;
        let z = imod(pos - context.player_pos.z + (context.width - 1)/2, context.width)  - (context.width - 1)/2 + 128;
        let offset_pos = global_id.x * 7u + global_id.y;//location for a specific face;
        commands.data[id].vertex_count = u32(chunk_table.data[offset_pos+1u]-chunk_table.data[offset_pos])*6u;
        commands.data[id].instance_count = 1u;
        commands.data[id].base_vertex = chunk_table.data[offset_pos]*6u;
        commands.data[id].base_instance = insertBits(insertBits(u32(x),u32(y),8u,8u),u32(z),16u,8u);
    }
}
