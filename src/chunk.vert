#version 460

layout(set = 0, binding = 0) uniform Uniform{mat4 view_proj;};
struct Face
{
    int pos_dir;
    int texture;
    int light;
};
layout(set = 1,binding = 0) readonly restrict buffer faces_buffer
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

const int INDICES[6] = int[6](0,1,2,2,3,0);

vec3 face_vertex(in int vertexID){
    return vec3(bitfieldExtract(0xCC69F0u,vertexID,1),bitfieldExtract(0xCC69F0u,vertexID,1),bitfieldExtract(0xCC69F0u,vertexID,1));
}

void main()
{
    int face_vertex_id=INDICES[int(mod(gl_VertexIndex,6))];
    int face_id=gl_VertexIndex/6;
    int pos_dir=faces_data[face_id+gl_BaseVertex].pos_dir;
    vec4 pos = (unpackUnorm4x8(gl_BaseInstance) * 255 - 128)* 32;

    gl_Position = view_proj * (vec4(pos.xyz + vec3(pos_dir&0x000000FF,bitfieldExtract(pos_dir,8,8),bitfieldExtract(pos_dir,16,8)) + face_vertex(bitfieldExtract(pos_dir,24,8)*4+face_vertex_id), 1.0));
    tex_coord = uv[face_vertex_id];
    tex_id = faces_data[face_id].texture;
    light = faces_data[face_id].light;
}