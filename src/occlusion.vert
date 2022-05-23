#version 450
layout(set = 0, binding = 0) uniform Uniform{mat4 view_proj;};
layout(set = 1, binding = 0) readonly restrict buffer Position{uint boxes[];};
layout(location = 1) flat out ivec3 chunk_pos;
const int INDICES[6] = int[](0,1,2,2,3,0);
void main() {
    int face_vertex_id=INDICES[int(mod(gl_VertexIndex,6))];
    uint pos = boxes[gl_VertexIndex/18];
    chunk_pos.x = int(pos & 255u) - 128;
    chunk_pos.y = int(bitfieldExtract( pos, 8, 8)) - 128;
    chunk_pos.z = int(bitfieldExtract( pos, 16, 8)) - 128;
    uint dir = uint(mod(gl_VertexIndex,18)/6);
    int face;
    if(dir==0)
    {
        if(chunk_pos.x<0){
            face=0;
        }
        else face=1;
    }
    else if(dir==1){
        if(chunk_pos.z<0){
            face=2;
        }
        else face=3;
    }
    else if(chunk_pos.y<0){
        face=4;
    }
    else face=5;
    face_vertex_id = face * 6 + face_vertex_id;
    gl_Position = vec4(bitfieldExtract(0xCC69F0u,face_vertex_id,1),bitfieldExtract(0xCC69F0u,face_vertex_id,1),bitfieldExtract(0xCC69F0u,face_vertex_id,1), 1.0);
}
