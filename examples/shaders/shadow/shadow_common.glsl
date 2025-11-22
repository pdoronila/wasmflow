// Shadow Sampling Common Functions
// Provides PCF (Percentage Closer Filtering) utilities for soft shadows

#version 450

// Constants
const int PCF_SAMPLES_4 = 4;
const int PCF_SAMPLES_9 = 9;
const int PCF_SAMPLES_16 = 16;

/// PCF with 4 samples (2×2 pattern)
/// Returns shadow factor [0.0 = fully shadowed, 1.0 = fully lit]
float pcf4(sampler2DShadow shadowMap, vec3 shadowCoord, vec2 texelSize) {
    float shadow = 0.0;

    // 2×2 sample pattern
    for (int x = -1; x <= 0; x++) {
        for (int y = -1; y <= 0; y++) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            vec3 sampleCoord = vec3(shadowCoord.xy + offset, shadowCoord.z);
            shadow += texture(shadowMap, sampleCoord);
        }
    }

    return shadow / 4.0;
}

/// PCF with 9 samples (3×3 pattern)
/// Returns shadow factor [0.0 = fully shadowed, 1.0 = fully lit]
float pcf9(sampler2DShadow shadowMap, vec3 shadowCoord, vec2 texelSize) {
    float shadow = 0.0;

    // 3×3 sample pattern
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            vec3 sampleCoord = vec3(shadowCoord.xy + offset, shadowCoord.z);
            shadow += texture(shadowMap, sampleCoord);
        }
    }

    return shadow / 9.0;
}

/// PCF with 16 samples (4×4 pattern)
/// Returns shadow factor [0.0 = fully shadowed, 1.0 = fully lit]
float pcf16(sampler2DShadow shadowMap, vec3 shadowCoord, vec2 texelSize) {
    float shadow = 0.0;

    // 4×4 sample pattern
    for (int x = -2; x <= 1; x++) {
        for (int y = -2; y <= 1; y++) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            vec3 sampleCoord = vec3(shadowCoord.xy + offset, shadowCoord.z);
            shadow += texture(shadowMap, sampleCoord);
        }
    }

    return shadow / 16.0;
}

/// Simple shadow test (single sample, hard shadows)
/// Returns shadow factor [0.0 = fully shadowed, 1.0 = fully lit]
float shadowTest(sampler2DShadow shadowMap, vec3 shadowCoord) {
    return texture(shadowMap, shadowCoord);
}

/// Shadow bias calculation to prevent shadow acne
/// Returns adjusted depth value for shadow comparison
float calculateShadowBias(vec3 normal, vec3 lightDir, float baseBias, float maxBias) {
    // Slope-scale bias: more bias for surfaces at grazing angles to light
    float cosTheta = max(dot(normal, lightDir), 0.0);
    float bias = baseBias + maxBias * (1.0 - cosTheta);
    return bias;
}
