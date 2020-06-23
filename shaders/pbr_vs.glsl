#version 450 

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec2 a_tex_coords;
layout (location = 2) in vec3 a_normal;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec3 world_pos;
layout(location=2) out vec3 normal;

layout(location=3) out flat int instance_index;

layout(set=0, binding=0)
uniform MvpUniforms {
    vec4 u_view_position;
    mat4 u_view_proj;
    mat4 u_model; // now using instanced 
};

layout(set=0, binding=2)
buffer Transforms {
    mat4 s_models[];
};

void main()
{
    mat4 s_model = s_models[gl_InstanceIndex];
    instance_index = gl_InstanceIndex;
    v_tex_coords = a_tex_coords;
    world_pos = vec3(s_model * vec4(a_position, 1.0));
    normal = mat3(transpose(inverse(s_model))) * a_normal;
    gl_Position = u_view_proj * vec4(world_pos, 1.0);
}

