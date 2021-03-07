#version 450

layout(location=0) in vec2 v_position;
layout(location=1) in vec4 src_rect;
layout(location=2) in vec4 dest_rect;
layout(location=3) in float alpha;

layout(set = 1, binding = 0)
uniform Uniforms {
    mat4 view;
};

layout(location=0) out vec2 tex_coords;
layout(location=1) out float tex_alpha;

void main() {
    tex_alpha = alpha;
    tex_coords = v_position * src_rect.zw + src_rect.xy;
    gl_Position = view * vec4(v_position * dest_rect.zw + dest_rect.xy, 0.0, 1.0);
}
