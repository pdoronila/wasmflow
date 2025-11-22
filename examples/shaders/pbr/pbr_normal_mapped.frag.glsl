#version 450

// Fragment shader for PBR with normal mapping
// Implements Cook-Torrance BRDF with normal map support

layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_view_dir;
layout(location = 4) in vec3 frag_tangent;
layout(location = 5) in vec3 frag_bitangent;

layout(set = 1, binding = 0) uniform MaterialUniforms {
    vec4 base_color;
    float metallic;
    float roughness;
    float ao;  // Ambient occlusion
    float normal_strength;  // Normal map intensity [0=no effect, 1=full effect]
};

layout(set = 1, binding = 1) uniform sampler2D normal_map;

layout(set = 2, binding = 0) uniform LightUniforms {
    vec3 light_direction;  // Directional light (normalized)
    vec3 light_color;
    float light_intensity;
};

layout(location = 0) out vec4 out_color;

const float PI = 3.14159265359;

// GGX/Trowbridge-Reitz normal distribution function
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return a2 / denom;
}

// Smith geometry function (GGX variant)
float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;

    // G1 for view direction
    float NdotV = max(dot(N, V), 0.0);
    float denom_v = NdotV + sqrt(a2 + (1.0 - a2) * NdotV * NdotV);
    float G1_V = (2.0 * NdotV) / denom_v;

    // G1 for light direction
    float NdotL = max(dot(N, L), 0.0);
    float denom_l = NdotL + sqrt(a2 + (1.0 - a2) * NdotL * NdotL);
    float G1_L = (2.0 * NdotL) / denom_l;

    return G1_V * G1_L;
}

// Fresnel-Schlick approximation
vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}

// Sample and convert normal map to world space
vec3 get_normal_from_map() {
    // Sample normal map
    vec3 tangent_normal = texture(normal_map, frag_uv).xyz;

    // Convert from [0,1] to [-1,1] range
    tangent_normal = tangent_normal * 2.0 - 1.0;

    // Apply normal strength (allows blending between flat and full bump)
    tangent_normal.xy *= normal_strength;

    // Renormalize
    tangent_normal = normalize(tangent_normal);

    // Construct TBN matrix
    vec3 T = normalize(frag_tangent);
    vec3 B = normalize(frag_bitangent);
    vec3 N = normalize(frag_normal);
    mat3 TBN = mat3(T, B, N);

    // Transform to world space
    return normalize(TBN * tangent_normal);
}

void main() {
    // Get normal from normal map
    vec3 N = get_normal_from_map();

    // Normalize interpolated vectors
    vec3 V = normalize(frag_view_dir);
    vec3 L = normalize(-light_direction);
    vec3 H = normalize(V + L);

    // Calculate F0 (base reflectivity)
    // Dielectrics: 0.04, Metals: base_color
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, base_color.rgb, metallic);

    // Cook-Torrance BRDF
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);

    // Calculate PBR terms
    float D = distribution_ggx(N, H, roughness);
    float G = geometry_smith(N, V, L, roughness);
    vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    // Specular BRDF
    vec3 numerator = D * G * F;
    float denominator = 4.0 * NdotV * NdotL + 0.0001; // Prevent division by zero
    vec3 specular = numerator / denominator;

    // Diffuse BRDF (Lambertian with energy conservation)
    vec3 kS = F;  // Specular contribution
    vec3 kD = vec3(1.0) - kS;  // Remaining energy for diffuse
    kD *= 1.0 - metallic;  // Metals have no diffuse

    vec3 diffuse = kD * base_color.rgb / PI;

    // Combine diffuse and specular
    vec3 radiance = light_color * light_intensity;
    vec3 Lo = (diffuse + specular) * radiance * NdotL;

    // Ambient lighting (simplified)
    vec3 ambient = vec3(0.03) * base_color.rgb * ao;

    vec3 final_color = ambient + Lo;

    // Tone mapping (Reinhard)
    final_color = final_color / (final_color + vec3(1.0));

    // Gamma correction
    final_color = pow(final_color, vec3(1.0/2.2));

    out_color = vec4(final_color, base_color.a);
}
