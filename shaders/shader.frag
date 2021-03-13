#version 450

layout(location=0) in vec2 tex_coords;
layout(location=1) in float tex_alpha;
layout(location=0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform texture2D tex_view;
layout(set = 0, binding = 1) uniform sampler tex_sampler;

void main() {
    vec4 tex_color = texture(sampler2D(tex_view, tex_sampler), tex_coords);
    frag_color = vec4(tex_color.xyz, tex_color.w * tex_alpha);
}
