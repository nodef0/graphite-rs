#version 450
layout (location = 0) in  vec3 a_position;
layout (location = 0) out vec3 v_position;

layout(set=0, binding=0)
uniform ViewProjUniform {
    mat4 u_view_proj;
}

void main() {
    v_position = a_position;
    gl_Position = u_view_proj * vec4(v_position, 1.0);
}
