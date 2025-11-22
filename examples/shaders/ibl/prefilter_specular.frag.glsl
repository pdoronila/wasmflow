// IBL Specular Pre-filter Shader
// Pre-filters environment map for different roughness levels using GGX

#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 localPos;

// Outputs
layout(location = 0) out vec4 outColor;

// Environment cubemap to pre-filter
layout(set = 0, binding = 0) uniform samplerCube u_envMap;

// Roughness level for this mip
layout(set = 0, binding = 1) uniform PrefilterParams {
    float roughness;
    float resolution; // Cubemap face resolution
} u_params;

// Constants
const float PI = 3.14159265359;
const uint SAMPLE_COUNT = 1024u;

// Low-discrepancy sequence (Hammersley)
float radicalInverse_VdC(uint bits) {
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}

vec2 hammersley(uint i, uint N) {
    return vec2(float(i) / float(N), radicalInverse_VdC(i));
}

// GGX importance sampling
vec3 importanceSampleGGX(vec2 Xi, vec3 N, float roughness) {
    float a = roughness * roughness;

    // Sample in spherical coordinates
    float phi = 2.0 * PI * Xi.x;
    float cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
    float sinTheta = sqrt(1.0 - cosTheta * cosTheta);

    // Spherical to cartesian (in tangent space)
    vec3 H = vec3(
        cos(phi) * sinTheta,
        sin(phi) * sinTheta,
        cosTheta
    );

    // Tangent space to world space
    vec3 up = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent = normalize(cross(up, N));
    vec3 bitangent = cross(N, tangent);

    return normalize(tangent * H.x + bitangent * H.y + N * H.z);
}

// GGX normal distribution function
float distributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

void main() {
    vec3 N = normalize(localPos);

    // Assume view direction equals reflection vector for integration
    vec3 R = N;
    vec3 V = R;

    vec3 prefilteredColor = vec3(0.0);
    float totalWeight = 0.0;

    for (uint i = 0u; i < SAMPLE_COUNT; ++i) {
        // Generate sample using Hammersley sequence
        vec2 Xi = hammersley(i, SAMPLE_COUNT);
        vec3 H = importanceSampleGGX(Xi, N, u_params.roughness);
        vec3 L = normalize(2.0 * dot(V, H) * H - V);

        float NdotL = max(dot(N, L), 0.0);

        if (NdotL > 0.0) {
            // Sample environment map
            // For rougher surfaces, sample from lower mip levels
            float D = distributionGGX(N, H, u_params.roughness);
            float NdotH = max(dot(N, H), 0.0);
            float HdotV = max(dot(H, V), 0.0);
            float pdf = D * NdotH / (4.0 * HdotV) + 0.0001;

            float saTexel = 4.0 * PI / (6.0 * u_params.resolution * u_params.resolution);
            float saSample = 1.0 / (float(SAMPLE_COUNT) * pdf + 0.0001);

            float mipLevel = u_params.roughness == 0.0 ? 0.0 : 0.5 * log2(saSample / saTexel);

            prefilteredColor += textureLod(u_envMap, L, mipLevel).rgb * NdotL;
            totalWeight += NdotL;
        }
    }

    prefilteredColor = prefilteredColor / totalWeight;

    outColor = vec4(prefilteredColor, 1.0);
}
