#version 450

// Fragment shader for PBR with multiple lights (directional, point, spot)
// Implements Cook-Torrance BRDF with GGX distribution

layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_view_dir;
layout(location = 4) in vec3 frag_tangent;

layout(set = 1, binding = 0) uniform MaterialUniforms {
    vec4 base_color;
    float metallic;
    float roughness;
    float ao;  // Ambient occlusion
};

// Maximum 8 lights (matches MAX_LIGHTS in buffer.rs)
const uint MAX_LIGHTS = 8;
const uint LIGHT_TYPE_DIRECTIONAL = 0;
const uint LIGHT_TYPE_POINT = 1;
const uint LIGHT_TYPE_SPOT = 2;

struct LightData {
    vec3 position_or_direction;  // Position for point/spot, direction for directional
    uint light_type;
    vec3 color;
    float intensity;
    vec3 spot_direction;  // Only used for spot lights
    float radius;  // Attenuation radius for point/spot lights
    float inner_cone_angle;  // Cosine of inner angle for spot lights
    float outer_cone_angle;  // Cosine of outer angle for spot lights
    vec2 _padding;
};

layout(set = 2, binding = 0) uniform MultiLightUniforms {
    LightData lights[MAX_LIGHTS];
    uint light_count;
    vec3 _padding;
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
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// Cook-Torrance BRDF
vec3 cook_torrance_brdf(vec3 N, vec3 V, vec3 L, vec3 F0, float roughness, vec3 albedo, float metallic) {
    vec3 H = normalize(V + L);

    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);

    // Early exit if light or view is below surface
    if (NdotV <= 0.0 || NdotL <= 0.0) {
        return vec3(0.0);
    }

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

    vec3 diffuse = kD * albedo / PI;

    return diffuse + specular;
}

// Calculate contribution from a single light
vec3 calculate_light(LightData light, vec3 N, vec3 V, vec3 world_pos, vec3 F0, float roughness, vec3 albedo, float metallic) {
    vec3 L;
    float attenuation = 1.0;

    if (light.light_type == LIGHT_TYPE_DIRECTIONAL) {
        // Directional light: direction is constant
        L = normalize(-light.position_or_direction);

    } else if (light.light_type == LIGHT_TYPE_POINT) {
        // Point light: calculate direction from surface to light
        vec3 light_to_surface = world_pos - light.position_or_direction;
        float distance = length(light_to_surface);
        L = -normalize(light_to_surface);

        // Inverse square falloff with radius
        attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));
        attenuation = clamp(attenuation, 0.0, 1.0);

    } else if (light.light_type == LIGHT_TYPE_SPOT) {
        // Spot light: calculate direction and cone attenuation
        vec3 light_to_surface = world_pos - light.position_or_direction;
        float distance = length(light_to_surface);
        L = -normalize(light_to_surface);

        // Distance attenuation (same as point light)
        float distance_attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));

        // Cone attenuation (smooth falloff between inner and outer angles)
        vec3 spot_dir = normalize(light.spot_direction);
        float cos_angle = dot(normalize(light_to_surface), spot_dir);
        float cone_attenuation = smoothstep(
            light.outer_cone_angle,
            light.inner_cone_angle,
            cos_angle
        );

        attenuation = distance_attenuation * cone_attenuation;
        attenuation = clamp(attenuation, 0.0, 1.0);

    } else {
        // Unknown light type
        return vec3(0.0);
    }

    // Calculate BRDF
    vec3 brdf = cook_torrance_brdf(N, V, L, F0, roughness, albedo, metallic);

    // Calculate radiance
    vec3 radiance = light.color * light.intensity * attenuation;

    // Calculate outgoing radiance
    float NdotL = max(dot(N, L), 0.0);
    return brdf * radiance * NdotL;
}

void main() {
    // Normalize interpolated vectors
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(frag_view_dir);

    // Calculate F0 (base reflectivity)
    // Dielectrics: 0.04, Metals: base_color
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, base_color.rgb, metallic);

    // Accumulate lighting from all lights
    vec3 Lo = vec3(0.0);
    for (uint i = 0; i < light_count && i < MAX_LIGHTS; i++) {
        Lo += calculate_light(lights[i], N, V, frag_world_pos, F0, roughness, base_color.rgb, metallic);
    }

    // Ambient lighting (simplified)
    vec3 ambient = vec3(0.03) * base_color.rgb * ao;

    vec3 final_color = ambient + Lo;

    // Tone mapping (Reinhard)
    final_color = final_color / (final_color + vec3(1.0));

    // Gamma correction
    final_color = pow(final_color, vec3(1.0/2.2));

    out_color = vec4(final_color, base_color.a);
}
