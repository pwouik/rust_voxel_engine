#version 450
layout(location = 0) in vec2 tex_coord;
layout(location = 1) flat in uint tex_id;
layout(set = 0, binding = 0) uniform texture2D textures[3];
layout(set = 0, binding = 1) uniform sampler textureSampler;

layout(location = 0) out vec4 out_color;

// Fonction main

void main()
{
    out_color = texture(sampler2D(textures[tex_id], textureSampler), tex_coord);
}