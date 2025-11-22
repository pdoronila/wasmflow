// Equirectangular to Cubemap Conversion
// Converts 2D equirectangular environment map to cubemap

#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 localPos;

// Outputs
layout(location = 0) out vec4 outColor;

// Equirectangular texture
layout(set = 0, binding = 0) uniform sampler2D u_equirectMap;

// Constants
const vec2 invAtan = vec2(0.1591, 0.3183);  // (1/(2*PI), 1/PI)

/// Convert 3D cubemap direction to 2D equirectangular UV
vec2 sampleEquirectangularMap(vec3 v) {
    // Convert cartesian to spherical coordinates
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));

    // Normalize to [0, 1] range
    uv *= invAtan;
    uv += 0.5;

    return uv;
}

void main() {
    // Sample direction from cubemap position
    vec3 direction = normalize(localPos);

    // Convert to equirectangular UV
    vec2 uv = sampleEquirectangularMap(direction);

    // Sample equirectangular map
    vec3 color = texture(u_equirectMap, uv).rgb;

    outColor = vec4(color, 1.0);
}
