[[block]]
struct Blocks {
    data: [[stride(4)]] array<u32>;
};
struct Face {
    pos_dir:u32;
    texture:u32;
    light:u32;
};
[[block]]
struct Mesh {
    data: [[stride(12)]] array<Face>;
};

[[block]]
struct Counter {
    c: atomic<u32>;
};


[[group(0), binding(0)]]
var<storage, read> blocks: Blocks;
[[group(0), binding(1)]]
var<storage, write> mesh: Mesh;
[[group(0), binding(2)]]
var<storage, read_write> counter: Counter;

var<workgroup> workgroup_counter:atomic<u32>;
var<workgroup> workgroup_index:u32;

var<private> TEXTURE_INDEX: array<u32,18> = array<u32,18>(0u,0u,0u,0u,2u,1u,2u,2u,2u,2u,2u,2u,3u,3u,3u,3u,3u,3u);


[[stage(compute), workgroup_size(32,32,1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>,[[builtin(local_invocation_index)]] thread_index:u32) {
    if(thread_index == 0u){
        atomicStore(&workgroup_counter,0u);
    }
    let block_pos:u32 = global_id.x + 34u * global_id.z + 34u * 34u * global_id.y + 1u + 34u + 34u * 34u;
    let block:u32 = blocks.data[block_pos];
    let blockX:u32 = blocks.data[block_pos + 1u];
    let blockZ:u32 = blocks.data[block_pos + 34u];
    let blockY:u32 = blocks.data[block_pos + 34u * 34u];
    var local_counter:u32 = 0u;
    var predicate:array<bool,6> = array<bool,6>(false,false,false,false,false,false);
    if(block>0u && blockX==0u){
        local_counter = local_counter + 1u;
        predicate[0u] = true;
    }
    else{
        if(block==0u && blockX>0u){
            local_counter = local_counter + 1u;
            predicate[1u] = true;
        }
    }
    if(block>0u && blockZ==0u){
        local_counter = local_counter + 1u;
        predicate[2u] = true;
    }
    else{
        if(block==0u && blockZ>0u){
            local_counter = local_counter + 1u;
            predicate[3u] = true;
        }
    }
    if(block>0u && blockY==0u){
        local_counter = local_counter + 1u;
        predicate[4u] = true;
    }
    else{
        if(block==0u && blockY>0u){
            local_counter = local_counter + 1u;
            predicate[5u] = true;
        }
    }
    let local_index = atomicAdd(&workgroup_counter,local_counter);
    workgroupBarrier();
    if(thread_index == 0u){
        workgroup_index = atomicAdd(&counter.c,workgroup_counter);
    }
    workgroupBarrier();
    var index = workgroup_index + local_index;
    if(predicate[0u]){
        mesh.data[index].pos_dir=global_id.x + ( global_id.y << 8u ) + ( global_id.z << 16u ) + ( 1u << 24u );
        mesh.data[index].texture=TEXTURE_INDEX[block * 6u - 5u];
        mesh.data[index].light=0xFFFFFFFFu;
        index = index + 1u;
    }
    if(predicate[1u]){
        mesh.data[index].pos_dir=global_id.x + 1u + ( global_id.y << 8u ) + ( global_id.z << 16u );
        mesh.data[index].texture=TEXTURE_INDEX[blockX * 6u - 6u];
        mesh.data[index].light=0xFFFFFFFFu;
        index = index + 1u;
    }
    if(predicate[2u]){
        mesh.data[index].pos_dir=global_id.x + ( global_id.y << 8u ) + ( global_id.z << 16u ) + ( 3u << 24u );
        mesh.data[index].texture=TEXTURE_INDEX[block * 6u - 3u];
        mesh.data[index].light=0xFFFFFFFFu;
        index = index + 1u;
    }
    if(predicate[3u]){
        mesh.data[index].pos_dir=global_id.x + ( global_id.y << 8u ) + (( global_id.z + 1u ) << 16u ) + ( 2u << 24u);
        mesh.data[index].texture=TEXTURE_INDEX[blockZ * 6u - 4u];
        mesh.data[index].light=0xFFFFFFFFu;
        index = index + 1u;
    }
    if(predicate[4u]){
        mesh.data[index].pos_dir=global_id.x + ( global_id.y << 8u ) + ( global_id.z << 16u ) + ( 5u << 24u );
        mesh.data[index].texture=TEXTURE_INDEX[block * 6u - 1u];
        mesh.data[index].light=0xFFFFFFFFu;
        index = index + 1u;
    }
    if(predicate[5u]){
        mesh.data[index].pos_dir=global_id.x + (( global_id.y + 1u ) << 8u ) + ( global_id.z << 16u ) + ( 4u << 24u );
        mesh.data[index].texture=TEXTURE_INDEX[blockY * 6u - 2u];
        mesh.data[index].light=0xFFFFFFFFu;
    }
}