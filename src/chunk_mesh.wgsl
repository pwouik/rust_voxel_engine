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

var<private> TEXTURE_INDEX: array<u32,18> = array<u32,18>(0u,0u,0u,0u,2u,1u,2u,2u,2u,2u,2u,2u,3u,3u,3u,3u,3u,3u);


[[stage(compute), workgroup_size(32,32,1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let block1_pos:u32 = global_id.x + 34u * global_id.y + 34u * 34u * global_id.z + 1u + 34u + 34u * 34u;
    let block1 = blocks.data[block1_pos];
    let block2 = blocks.data[block1_pos + 1u];
    if(block1>0u && block2==0u){
        let index=atomicAdd(&counter.c,1u);
        mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u;
        mesh.data[index].texture=TEXTURE_INDEX[block1 * 6u - 5u];
        mesh.data[index].light=0xFFFFFFFFu;
    }
    else{
        if(block1==0u && block2>0u){
            let index=atomicAdd(&counter.c,1u);
            mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u + 1u << 24u;
            mesh.data[index].texture=TEXTURE_INDEX[block2 * 6u - 6u];
            mesh.data[index].light=0xFFFFFFFFu;
        }
    }
    let block2 = blocks.data[block1_pos + 34u];
    if(block1>0u && block2==0u){
        let index=atomicAdd(&counter.c,1u);
        mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u + 2u << 24u;
        mesh.data[index].texture=TEXTURE_INDEX[block1 * 6u - 3u];
        mesh.data[index].light=0xFFFFFFFFu;
    }
    else{
        if(block1==0u && block2>0u){
            let index=atomicAdd(&counter.c,1u);
            mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u + 3u << 24u;
            mesh.data[index].texture=TEXTURE_INDEX[block2 * 6u - 4u];
            mesh.data[index].light=0xFFFFFFFFu;
        }
    }
    let block2 = blocks.data[block1_pos + 34u * 43u];
    if(block1>0u && block2==0u){
        let index=atomicAdd(&counter.c,1u);
        mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u + 4u << 24u;
        mesh.data[index].texture=TEXTURE_INDEX[block1 * 6u - 2u];
        mesh.data[index].light=0xFFFFFFFFu;
    }
    else{
        if(block1==0u && block2>0u){
            let index=atomicAdd(&counter.c,1u);
            mesh.data[index].pos_dir=global_id.x + global_id.y << 8u + global_id.z << 16u + 5u << 24u;
            mesh.data[index].texture=TEXTURE_INDEX[block2 * 6u - 1u];
            mesh.data[index].light=0xFFFFFFFFu;
        }
    }
}