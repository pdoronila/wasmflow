// Skybox Fragment Shader
// Samples environment cubemap for background rendering

#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 texCoord;  // Cubemap sample direction

// Outputs
layout(location = 0) out vec4 outColor;

// Environment cubemap
layout(set = 0, binding = 1) uniform samplerCube u_skybox;

// Optional exposure/gamma correction
layout(set = 0, binding = 2) uniform SkyboxParams {
    float exposure;       // HDR exposure (default: 1.0)
    float gamma;          // Gamma correction (default: 2.2)
    float brightness;     // Brightness multiplier (default: 1.0)
    uint enableToneMap;   // 0 = off, 1 = on (default: 0)
} u_params;

/// Simple Reinhard tone mapping
vec3 toneMapReinhard(vec3 color) {
    return color / (color + vec3(1.0));
}

/// ACES filmic tone mapping (approximation)
vec3 toneMapACES(vec3 color) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

void main() {
    // Sample cubemap
    vec3 color = texture(u_skybox, texCoord).rgb;

    // Apply exposure
    color *= u_params.exposure * u_params.brightness;

    // Optional tone mapping (for HDR environment maps)
    if (u_params.enableToneMap == 1u) {
        color = toneMapACES(color);
    }

    // Gamma correction
    color = pow(color, vec3(1.0 / u_params.gamma));

    outColor = vec4(color, 1.0);
}
