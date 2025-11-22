// Point Light Shadow Fragment Shader
// Uses cubemap shadow mapping with PCF filtering

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

// Point light uniforms
layout(set = 0, binding = 1) uniform PointLight {
    vec3 position;
    vec3 color;
    float intensity;
    float radius;       // Light attenuation radius
    float farPlane;     // Shadow far plane (should match radius)
    float shadowBias;   // Base shadow bias (e.g., 0.05)
} u_light;

// Shadow cubemap
layout(set = 0, binding = 2) uniform samplerCubeShadow shadowCubemap;

// Material
layout(set = 0, binding = 3) uniform Material {
    vec3 baseColor;
    float roughness;
} u_material;

/// PCF for cubemap shadows (6 samples)
/// Samples neighboring cubemap texels for soft shadows
float pcfCubemap(vec3 lightToFrag, float currentDepth, float bias) {
    float shadow = 0.0;
    float samples = 6.0;

    // Sample offsets for PCF
    vec3 sampleOffsets[6] = vec3[](
        vec3(1, 1, 0), vec3(1, -1, 0), vec3(-1, 1, 0),
        vec3(-1, -1, 0), vec3(0, 1, 1), vec3(0, -1, 1)
    );

    float diskRadius = 0.05;  // PCF sample radius

    for (int i = 0; i < 6; i++) {
        vec3 sampleDir = lightToFrag + sampleOffsets[i] * diskRadius;
        float shadowDepth = texture(shadowCubemap, vec4(sampleDir, currentDepth - bias));
        shadow += shadowDepth;
    }

    return shadow / samples;
}

/// Calculate point light shadow factor
float calculateShadow(vec3 worldPos, vec3 normal) {
    // Vector from fragment to light
    vec3 fragToLight = worldPos - u_light.position;
    float currentDepth = length(fragToLight);

    // Early out if beyond light radius
    if (currentDepth > u_light.farPlane) {
        return 0.0;  // Outside light range = fully shadowed
    }

    // Normalize depth to [0, 1] range
    float normalizedDepth = currentDepth / u_light.farPlane;

    // Calculate bias
    vec3 lightDir = normalize(-fragToLight);
    float cosTheta = max(dot(normal, lightDir), 0.0);
    float bias = u_light.shadowBias * (1.0 - cosTheta);

    // Sample cubemap with PCF
    float shadowFactor = pcfCubemap(fragToLight, normalizedDepth, bias);

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

    // Diffuse lighting
    float diffuse = max(dot(normal, lightDir), 0.0);

    // Shadow calculation
    float shadowFactor = calculateShadow(fragPosition, normal);

    // Final color with attenuation
    vec3 lighting = u_material.baseColor * u_light.color * u_light.intensity * diffuse * attenuation * shadowFactor;

    // Ambient term
    vec3 ambient = u_material.baseColor * 0.1;

    outColor = vec4(lighting + ambient, 1.0);
}
