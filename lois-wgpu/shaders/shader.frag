#version 450

layout(set = 0, binding = 0) uniform texture2D tex_view;
layout(set = 0, binding = 1) uniform sampler tex_sampler;

layout(location=0) in vec2 tex_coords;

layout(location=0) out vec4 frag_color;

void main() {
    frag_color = texture(sampler2D(tex_view, tex_sampler), tex_coords);
}
