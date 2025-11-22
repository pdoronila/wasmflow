// Directional Light Shadow Fragment Shader
// Supports cascaded shadow maps (CSM) with PCF filtering

#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;

// Outputs
layout(location = 0) out vec4 outColor;

// Camera uniforms
layout(set = 0, binding = 0) uniform Camera {
    mat4 viewMatrix;
    mat4 projectionMatrix;
    vec3 cameraPosition;
} u_camera;

// Shadow uniforms
layout(set = 0, binding = 1) uniform DirectionalShadow {
    mat4 shadowMatrices[4];  // Up to 4 cascades
    vec4 cascadeSplits;      // Split distances (x, y, z, w)
    uint cascadeCount;       // Number of active cascades
    float shadowBias;        // Base shadow bias (e.g., 0.005)
    float maxBias;           // Max bias for grazing angles (e.g., 0.05)
} u_shadow;

// Light uniforms
layout(set = 0, binding = 2) uniform Light {
    vec3 direction;  // Light direction (normalized)
    vec3 color;
    float intensity;
} u_light;

// Shadow map array (one texture per cascade)
layout(set = 0, binding = 3) uniform sampler2DShadow shadowMaps[4];

// Material
layout(set = 0, binding = 4) uniform Material {
    vec3 baseColor;
    float roughness;
} u_material;

/// Select cascade based on view-space depth
uint selectCascade(float viewSpaceDepth) {
    for (uint i = 0u; i < u_shadow.cascadeCount; i++) {
        if (viewSpaceDepth < u_shadow.cascadeSplits[i]) {
            return i;
        }
    }
    return u_shadow.cascadeCount - 1u;
}

/// PCF shadow sampling (9 samples)
float pcf9(sampler2DShadow shadowMap, vec3 shadowCoord) {
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    float shadow = 0.0;

    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            shadow += texture(shadowMap, vec3(shadowCoord.xy + offset, shadowCoord.z));
        }
    }

    return shadow / 9.0;
}

/// Calculate shadow factor
float calculateShadow(vec3 worldPos, vec3 normal) {
    // Transform to view space to select cascade
    vec4 viewPos = u_camera.viewMatrix * vec4(worldPos, 1.0);
    float viewDepth = -viewPos.z;

    // Select cascade
    uint cascadeIndex = selectCascade(viewDepth);

    // Transform to shadow space
    vec4 shadowPos = u_shadow.shadowMatrices[cascadeIndex] * vec4(worldPos, 1.0);
    vec3 shadowCoord = shadowPos.xyz / shadowPos.w;

    // Convert to [0, 1] range
    shadowCoord = shadowCoord * 0.5 + 0.5;

    // Check if outside shadow map bounds
    if (shadowCoord.x < 0.0 || shadowCoord.x > 1.0 ||
        shadowCoord.y < 0.0 || shadowCoord.y > 1.0 ||
        shadowCoord.z < 0.0 || shadowCoord.z > 1.0) {
        return 1.0;  // Outside shadow map = fully lit
    }

    // Calculate bias to prevent shadow acne
    float cosTheta = max(dot(normal, -u_light.direction), 0.0);
    float bias = u_shadow.shadowBias + u_shadow.maxBias * (1.0 - cosTheta);
    shadowCoord.z -= bias;

    // Sample shadow map with PCF
    float shadowFactor = pcf9(shadowMaps[cascadeIndex], shadowCoord);

    return shadowFactor;
}

void main() {
    vec3 normal = normalize(fragNormal);
    vec3 lightDir = -u_light.direction;

    // Diffuse lighting
    float diffuse = max(dot(normal, lightDir), 0.0);

    // Shadow calculation
    float shadowFactor = calculateShadow(fragPosition, normal);

    // Final color
    vec3 lighting = u_material.baseColor * u_light.color * u_light.intensity * diffuse * shadowFactor;

    // Add ambient term (10% to prevent pure black shadows)
    vec3 ambient = u_material.baseColor * 0.1;

    outColor = vec4(lighting + ambient, 1.0);
}
