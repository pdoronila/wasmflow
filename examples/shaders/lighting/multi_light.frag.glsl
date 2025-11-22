#version 450

// Fragment shader for multiple lights (directional + point)

layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_view_dir;

layout(set = 1, binding = 0) uniform MaterialUniforms {
    vec4 base_color;
    float metallic;
    float roughness;
};

// Maximum 8 lights (matches MAX_LIGHTS in buffer.rs)
const uint MAX_LIGHTS = 8;
const uint LIGHT_TYPE_DIRECTIONAL = 0;
const uint LIGHT_TYPE_POINT = 1;

struct Light {
    vec3 position_or_direction;
    uint light_type;
    vec3 color;
    float intensity;
    float radius;
    vec3 _padding;
};

layout(set = 2, binding = 0) uniform MultiLightUniforms {
    Light lights[MAX_LIGHTS];
    uint light_count;
    vec3 _padding;
};

layout(location = 0) out vec4 out_color;

// Calculate contribution from a single light
vec3 calculate_light(Light light, vec3 N, vec3 V, vec3 world_pos) {
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

        // Attenuation based on distance and radius
        attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));
        attenuation = clamp(attenuation, 0.0, 1.0);
    } else {
        // Unknown light type
        return vec3(0.0);
    }

    // Diffuse component
    float diffuse = max(dot(N, L), 0.0);

    // Specular component (Phong)
    vec3 R = reflect(-L, N);
    float spec_angle = max(dot(R, V), 0.0);
    float shininess = mix(128.0, 1.0, roughness);
    float specular = pow(spec_angle, shininess);

    // Combine components with attenuation
    vec3 diffuse_color = base_color.rgb * light.color * diffuse * light.intensity * attenuation;
    vec3 specular_color = light.color * specular * light.intensity * (1.0 - roughness) * attenuation;

    return diffuse_color + specular_color;
}

void main() {
    // Normalize interpolated vectors
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(frag_view_dir);

    // Start with ambient lighting
    vec3 final_color = base_color.rgb * 0.1;

    // Accumulate contributions from all active lights
    for (uint i = 0; i < light_count && i < MAX_LIGHTS; i++) {
        final_color += calculate_light(lights[i], N, V, frag_world_pos);
    }

    out_color = vec4(final_color, base_color.a);
}
