// Cubemap Convolution Vertex Shader
// Used for irradiance and pre-filter passes

#version 450

// Vertex position (cube vertices)
layout(location = 0) in vec3 position;

// Outputs to fragment shader
layout(location = 0) out vec3 localPos;

// View and projection matrices for current cubemap face
layout(set = 0, binding = 2) uniform CubemapMatrices {
    mat4 viewMatrices[6];     // 6 view matrices (one per face)
    mat4 projectionMatrix;    // Same projection for all faces
    uint faceIndex;           // Current face being rendered
} u_matrices;

void main() {
    localPos = position;

    mat4 view = u_matrices.viewMatrices[u_matrices.faceIndex];
    gl_Position = u_matrices.projectionMatrix * view * vec4(localPos, 1.0);
}
