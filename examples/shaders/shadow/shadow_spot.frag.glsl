// Spot Light Shadow Fragment Shader
// Uses perspective shadow mapping with PCF filtering

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

// Spot light uniforms
layout(set = 0, binding = 1) uniform SpotLight {
    vec3 position;
    vec3 direction;     // Normalized direction
    vec3 color;
    float intensity;
    float innerAngle;   // Inner cone angle (radians)
    float outerAngle;   // Outer cone angle (radians)
    float radius;       // Light attenuation radius
    mat4 shadowMatrix;  // Shadow view-projection matrix
    float shadowBias;   // Base shadow bias (e.g., 0.005)
    float maxBias;      // Max bias for grazing angles (e.g., 0.05)
} u_light;

// Shadow map
layout(set = 0, binding = 2) uniform sampler2DShadow shadowMap;

// Material
layout(set = 0, binding = 3) uniform Material {
    vec3 baseColor;
    float roughness;
} u_material;

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

/// Calculate cone attenuation
float calculateConeAttenuation(vec3 lightDir, vec3 spotDir) {
    float cosAngle = dot(-lightDir, spotDir);
    float innerCos = cos(u_light.innerAngle);
    float outerCos = cos(u_light.outerAngle);

    // Smooth transition between inner and outer cone
    float epsilon = innerCos - outerCos;
    float attenuation = clamp((cosAngle - outerCos) / epsilon, 0.0, 1.0);

    return attenuation;
}

/// Calculate spot light shadow factor
float calculateShadow(vec3 worldPos, vec3 normal) {
    // Transform to shadow space
    vec4 shadowPos = u_light.shadowMatrix * vec4(worldPos, 1.0);
    vec3 shadowCoord = shadowPos.xyz / shadowPos.w;

    // Convert to [0, 1] range
    shadowCoord = shadowCoord * 0.5 + 0.5;

    // Check if outside shadow map bounds
    if (shadowCoord.x < 0.0 || shadowCoord.x > 1.0 ||
        shadowCoord.y < 0.0 || shadowCoord.y > 1.0 ||
        shadowCoord.z < 0.0 || shadowCoord.z > 1.0) {
        return 1.0;  // Outside shadow map = fully lit
    }

    // Calculate bias
    vec3 lightDir = normalize(worldPos - u_light.position);
    float cosTheta = max(dot(normal, -lightDir), 0.0);
    float bias = u_light.shadowBias + u_light.maxBias * (1.0 - cosTheta);
    shadowCoord.z -= bias;

    // Sample shadow map with PCF
    float shadowFactor = pcf9(shadowMap, shadowCoord);

    return shadowFactor;
}

void main() {
    vec3 normal = normalize(fragNormal);

    // Vector from fragment to light
    vec3 fragToLight = u_light.position - fragPosition;
    float distance = length(fragToLight);
    vec3 lightDir = fragToLight / distance;

    // Distance attenuation
    float attenuation = 1.0 / (1.0 + (distance * distance) / (u_light.radius * u_light.radius));

    // Cone attenuation
    float coneAttenuation = calculateConeAttenuation(lightDir, u_light.direction);

    // Diffuse lighting
    float diffuse = max(dot(normal, lightDir), 0.0);

    // Shadow calculation
    float shadowFactor = calculateShadow(fragPosition, normal);

    // Final color with both attenuations
    vec3 lighting = u_material.baseColor * u_light.color * u_light.intensity *
                    diffuse * attenuation * coneAttenuation * shadowFactor;

    // Ambient term
    vec3 ambient = u_material.baseColor * 0.1;

    outColor = vec4(lighting + ambient, 1.0);
}
