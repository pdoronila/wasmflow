// PBR with Image-Based Lighting (IBL)
// Complete Cook-Torrance BRDF with environment lighting

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

// Material uniforms
layout(set = 0, binding = 1) uniform Material {
    vec3 baseColor;
    float metallic;
    float roughness;
    float ao;  // Ambient occlusion
} u_material;

// IBL textures
layout(set = 0, binding = 2) uniform samplerCube u_irradianceMap;      // Diffuse irradiance
layout(set = 0, binding = 3) uniform samplerCube u_prefilterMap;       // Specular pre-filtered
layout(set = 0, binding = 4) uniform sampler2D u_brdfLUT;              // BRDF integration LUT

// Optional direct lighting
layout(set = 0, binding = 5) uniform DirectLight {
    vec3 direction;
    vec3 color;
    float intensity;
    uint enabled;  // 0 = off, 1 = on
} u_directLight;

// Constants
const float PI = 3.14159265359;

// Fresnel-Schlick approximation
vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// Fresnel-Schlick with roughness for IBL
vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness) {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
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

    return nom / max(denom, 0.0000001);
}

// Smith geometry function
float geometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float geometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometrySchlickGGX(NdotV, roughness);
    float ggx1 = geometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(u_camera.cameraPosition - fragPosition);
    vec3 R = reflect(-V, N);

    // Calculate F0 (base reflectivity)
    vec3 F0 = vec3(0.04);  // Dielectric base
    F0 = mix(F0, u_material.baseColor, u_material.metallic);

    // Direct lighting (optional)
    vec3 Lo = vec3(0.0);
    if (u_directLight.enabled == 1u) {
        vec3 L = normalize(-u_directLight.direction);
        vec3 H = normalize(V + L);

        // Cook-Torrance BRDF
        float NDF = distributionGGX(N, H, u_material.roughness);
        float G = geometrySmith(N, V, L, u_material.roughness);
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        vec3 specular = numerator / denominator;

        // Energy conservation
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - u_material.metallic;

        float NdotL = max(dot(N, L), 0.0);
        Lo = (kD * u_material.baseColor / PI + specular) * u_directLight.color * u_directLight.intensity * NdotL;
    }

    // Ambient lighting (IBL)
    vec3 F = fresnelSchlickRoughness(max(dot(N, V), 0.0), F0, u_material.roughness);

    vec3 kS = F;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - u_material.metallic;

    // Diffuse IBL
    vec3 irradiance = texture(u_irradianceMap, N).rgb;
    vec3 diffuse = irradiance * u_material.baseColor;

    // Specular IBL
    const float MAX_REFLECTION_LOD = 4.0;  // Adjust based on mip levels
    vec3 prefilteredColor = textureLod(u_prefilterMap, R, u_material.roughness * MAX_REFLECTION_LOD).rgb;
    vec2 brdf = texture(u_brdfLUT, vec2(max(dot(N, V), 0.0), u_material.roughness)).rg;
    vec3 specular = prefilteredColor * (F * brdf.x + brdf.y);

    vec3 ambient = (kD * diffuse + specular) * u_material.ao;

    // Final color
    vec3 color = ambient + Lo;

    // HDR tone mapping (ACES)
    color = color / (color + vec3(1.0));

    // Gamma correction
    color = pow(color, vec3(1.0 / 2.2));

    outColor = vec4(color, 1.0);
}
