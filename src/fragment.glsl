#version 450
layout(location = 0) in vec2 tex_coord;
layout(location = 1) flat in uint tex_id;
layout(location = 2) flat in uint light;
layout(set = 0, binding = 0) uniform texture2D textures[3];
layout(set = 0, binding = 1) uniform sampler textureSampler;

layout(location = 0) out vec4 out_color;

// Fonction main

void main()
{
    vec4 lights=unpackUnorm4x8(light);
    out_color = texture(sampler2D(textures[tex_id], textureSampler), tex_coord)*mix(mix(lights[2],lights[3],tex_coord.x),mix(lights[1],lights[0],tex_coord.x),tex_coord.y);
}