// IBL Diffuse Irradiance Convolution Shader
// Generates diffuse irradiance map from environment cubemap

#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 localPos;

// Outputs
layout(location = 0) out vec4 outColor;

// Environment cubemap to convolve
layout(set = 0, binding = 0) uniform samplerCube u_envMap;

// Constants
const float PI = 3.14159265359;

void main() {
    // Sample direction is the normal for lambertian diffuse
    vec3 normal = normalize(localPos);

    // Tangent space calculation for hemisphere sampling
    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, normal));
    up = cross(normal, right);

    // Convolution: integrate over hemisphere
    vec3 irradiance = vec3(0.0);
    float sampleCount = 0.0;

    // Sample delta (lower = better quality, higher = faster)
    float sampleDelta = 0.025;

    for (float phi = 0.0; phi < 2.0 * PI; phi += sampleDelta) {
        for (float theta = 0.0; theta < 0.5 * PI; theta += sampleDelta) {
            // Spherical to cartesian (in tangent space)
            vec3 tangentSample = vec3(
                sin(theta) * cos(phi),
                sin(theta) * sin(phi),
                cos(theta)
            );

            // Tangent space to world space
            vec3 sampleVec = tangentSample.x * right +
                           tangentSample.y * up +
                           tangentSample.z * normal;

            // Sample environment map
            irradiance += texture(u_envMap, sampleVec).rgb * cos(theta) * sin(theta);
            sampleCount += 1.0;
        }
    }

    // Average and apply hemisphere integral factor
    irradiance = PI * irradiance / sampleCount;

    outColor = vec4(irradiance, 1.0);
}
