#version 450

layout(set = 1, binding = 0) uniform Uniform{mat4 view_proj;};
layout(push_constant) uniform Constant{ivec3 pos;};
layout(location = 0) in vec3 in_vertex;
layout(location = 1) in uint in_tex_id;

layout(location = 0) out vec2 tex_coord;
layout(location = 1) flat out uint tex_id;

const vec2 uv[4] = vec2[4](
    vec2(0.0,1.0),
    vec2(1.0,1.0),
    vec2(1.0,0.0),
    vec2(0.0,0.0));
void main()
{
    gl_Position = view_proj * (vec4(in_vertex+pos, 1.0));
    tex_coord = uv[int(mod(gl_VertexIndex,4))];
    tex_id = in_tex_id;
}