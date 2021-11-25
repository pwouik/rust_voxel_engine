#version 450

layout(set = 1, binding = 0) uniform Uniform{mat4 view_proj;};
layout(push_constant) uniform Constant{ivec3 pos;};
struct Face
{
    int pos_dir;
    int texture;
    int light;
};
layout(set = 2,binding = 0) readonly restrict buffer faces_buffer
{
    Face faces_data[];
};
layout(location = 0) out vec2 tex_coord;
layout(location = 1) flat out uint tex_id;
layout(location = 2) flat out uint light;

const vec2 uv[4] = vec2[4](
    vec2(1.0,1.0),
    vec2(0.0,1.0),
    vec2(0.0,0.0),
    vec2(1.0,0.0));
const vec3 FACES[24]=vec3[24](
    vec3(0.0, 0.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 1.0, 1.0),
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 0.0, 1.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(1.0, 1.0, 1.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 1.0),
    vec3(1.0, 1.0, 1.0),
    vec3(0.0, 1.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 1.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0),
    vec3(1.0, 1.0, 1.0),
    vec3(1.0, 1.0, 0.0)
);

const int INDICES[6] = int[6](0,1,2,2,3,0);

vec3 face_vertex(in uint vertexID){
    uint b = 1 << vertexID;
    return vec3((0xCC69F0u & b) != 0, (0xF0CCCCu & b) != 0, (0x69F096u & b) != 0);
}

void main()
{
    int face_vertex_id=INDICES[int(mod(gl_VertexIndex,6))];
    int face_id=gl_VertexIndex/6;
    int pos_dir=faces_data[face_id].pos_dir;
    gl_Position = view_proj * (vec4(pos+vec3(pos_dir&0x000000FF,pos_dir>>8&0x000000FF,pos_dir>>16&0x000000FF)+face_vertex((pos_dir>>24&0x000000FF)*4+face_vertex_id), 1.0));
    tex_coord = uv[face_vertex_id];
    tex_id = faces_data[face_id].texture;
    light = faces_data[face_id].light;
}