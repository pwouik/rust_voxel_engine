#version 450
layout(set = 0, binding = 0) uniform Uniform{mat4 view_proj;};
layout(set = 0, binding = 1) uniform Context{
    ivec3 player_pos;
    int height;
    int width;
};
layout(set = 1,binding = 1) restrict buffer State{uint visibility[];};//circular bounded
layout(location = 0) flat in ivec3 chunk_pos;
void main() {
    if(chunk_pos.x<=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))]=1;
    }
    if(chunk_pos.x>=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))+1]=1;
    }
    if(chunk_pos.y<=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))+2]=1;
    }
    if(chunk_pos.y>=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))+3]=1;
    }
    if(chunk_pos.z<=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))+4]=1;
    }
    if(chunk_pos.z>=0){
        visibility[uint(mod(player_pos.x+chunk_pos.x, width)+width*mod(player_pos.y+chunk_pos.y, height)+width*height*mod(player_pos.z+chunk_pos.z, width))+5]=1;
    }
}
