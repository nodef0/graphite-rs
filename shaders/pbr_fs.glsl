#version 450
layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec3 world_pos;
layout(location=2) in vec3 normal;

layout(location=3) in flat int instance_index;

layout(location=0) out vec4 frag_color;

layout(set=0, binding=0)
uniform MvpUniforms {
    vec4 u_view_position;
    mat4 u_view_proj;
    mat4 u_model;
};

layout(set = 0, binding = 1)
uniform PbrFragmentUniforms {
    vec4 albedo;
    vec4 light_positions[4];
    vec4 light_colors[4];
};

layout(set=0, binding=3)
buffer MaterialInfos {
    vec4 s_infos[];
};

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

layout(set = 1, binding = 2) uniform texture2D t_roughness;
layout(set = 1, binding = 3) uniform sampler s_roughness;

layout(set = 1, binding = 4) uniform texture2D t_ao;
layout(set = 1, binding = 5) uniform sampler s_ao;

layout(set = 1, binding = 6) uniform texture2D t_normal;
layout(set = 1, binding = 7) uniform sampler s_normal;

const float PI = 3.14159265359;

vec3 getNormalFromMap()
{
    vec3 tangentNormal = texture(sampler2D(t_normal, s_normal), v_tex_coords).xyz * 2.0 - 1.0;

    vec3 Q1  = dFdx(world_pos);
    vec3 Q2  = dFdy(world_pos);
    vec2 st1 = dFdx(v_tex_coords);
    vec2 st2 = dFdy(v_tex_coords);

    vec3 N   = normalize(normal);
    vec3 T   = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B   = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;

    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float GeometrySmith(float NdotV, float NdotL, float roughness)
{
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);

    return ggx1 * ggx2;
}

void main()
{
    vec4 info = s_infos[instance_index];
    float metallic = 1.0; // info.x;
    // float roughness = info.y;
    // float ambient_occlusion = info.z;
    
    vec3 albedo = pow(texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb, vec3(2.2));
    float roughness = texture(sampler2D(t_roughness, s_roughness), v_tex_coords).r;
    float ambient_occlusion = texture(sampler2D(t_ao, s_ao), v_tex_coords).r;

    vec3 N = getNormalFromMap(); // normalize(normal);
    vec3 V = normalize(vec3(u_view_position) - world_pos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    // reflectance equation
    vec3 Lo = vec3(0.0);
    for (int i = 0; i < 4; ++i)
    {
        // per-light radiance
        vec3 L = normalize(vec3(light_positions[i]) - world_pos);
        vec3 H = normalize(V + L);

        float distance    = length(vec3(light_positions[i]) - world_pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     = vec3(light_colors[i]) * attenuation;

        float NdotV = max(dot(N, V), 0.0);
        float NdotL = max(dot(N, L), 0.0);

        // Cook-Torrance BRDF
        float NDF = DistributionGGX(N, H, roughness);
        float G   = GeometrySmith(NdotV, NdotL, roughness);
        vec3 F    = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);
        
        vec3 num = NDF * G * F;
        float denom = 4.0 * NdotV * NdotL + 0.0001;
        vec3 specular = num / denom;

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        // add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // ambient, to be replace with IBL
    vec3 ambient = vec3(0.03) * albedo * ambient_occlusion;
    vec3 color = ambient + Lo;

    // HDR tonemap
    color = color / (color + vec3(1.0));
    // gamma correct
    color = pow(color, vec3(1.0/2.2));

    frag_color = vec4(color, 1.0);
}
