#version 450

// constants
layout(set = 1, binding = 0)
uniform Uniforms {
    mat4 view;
};

// in
layout(location=0) in vec2 v_position;
layout(location=1) in vec4 transform_col_1;
layout(location=2) in vec4 transform_col_2;
layout(location=3) in vec4 transform_col_3;
layout(location=4) in vec4 transform_col_4;
layout(location=5) in vec4 src_rect;

// out
layout(location=0) out vec2 tex_coords;

void main() {
    mat4 transform = mat4(
        transform_col_1,
        transform_col_2,
        transform_col_3,
        transform_col_4
    );
    tex_coords = v_position * src_rect.zw + src_rect.xy;
    gl_Position = view * transform * vec4(v_position, 0.0, 1.0);
}
