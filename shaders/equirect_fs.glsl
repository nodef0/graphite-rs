#version 450
layout (location = 0) in  v_position;
layout (location = 0) out f_color;

layout(set = 0, binding = 0) uniform texture2D t_equirect;
layout(set = 0, binding = 1) uniform sampler   s_equirect;

const vec2 inv_atan = vec2(0.1591, 0.3183);

vec2 sample_spherical_map(vec3 v) {
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= inv_atan;
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = sample_spherical_map(normalize(v_position));
    f_color = vec4(texture(sampler2D(t_equirect, s_equirect), uv).rgb, 1.0);
}
