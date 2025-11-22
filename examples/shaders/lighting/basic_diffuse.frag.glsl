#version 450

// Fragment shader for basic diffuse lighting (Lambert)

layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;

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
    // Normalize the interpolated normal
    vec3 N = normalize(frag_normal);

    // Light direction (pointing toward light source)
    vec3 L = normalize(-direction);

    // Calculate diffuse component using Lambert's cosine law
    float diffuse = max(dot(N, L), 0.0);

    // Combine material color with light
    vec3 lit_color = base_color.rgb * color * diffuse * intensity;

    // Add small ambient term to prevent pure black
    vec3 ambient = base_color.rgb * 0.1;

    out_color = vec4(lit_color + ambient, base_color.a);
}
