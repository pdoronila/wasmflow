#version 450

// Fragment shader for Phong lighting (diffuse + specular)

layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_view_dir;

layout(set = 1, binding = 0) uniform MaterialUniforms {
    vec4 base_color;
    float metallic;
    float roughness;
};

layout(set = 2, binding = 0) uniform LightUniforms {
    vec3 direction;
    float _padding1;
    vec3 color;
    float intensity;
};

layout(location = 0) out vec4 out_color;

void main() {
    // Normalize interpolated vectors
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(frag_view_dir);

    // Light direction (pointing toward light source)
    vec3 L = normalize(-direction);

    // Diffuse component (Lambert)
    float diffuse = max(dot(N, L), 0.0);

    // Specular component (Phong)
    vec3 R = reflect(-L, N);  // Reflect light direction around normal
    float spec_angle = max(dot(R, V), 0.0);

    // Convert roughness to shininess (inverse relationship)
    // Roughness 0.0 = very shiny (shininess 128)
    // Roughness 1.0 = very rough (shininess 1)
    float shininess = mix(128.0, 1.0, roughness);
    float specular = pow(spec_angle, shininess);

    // Combine lighting components
    vec3 diffuse_color = base_color.rgb * color * diffuse * intensity;
    vec3 specular_color = color * specular * intensity * (1.0 - roughness);

    // Add small ambient term
    vec3 ambient = base_color.rgb * 0.1;

    vec3 final_color = ambient + diffuse_color + specular_color;

    out_color = vec4(final_color, base_color.a);
}
