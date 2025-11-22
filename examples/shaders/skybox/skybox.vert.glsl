// Skybox Vertex Shader
// Renders environment cubemap as background

#version 450

// Vertex inputs (fullscreen quad vertices)
layout(location = 0) in vec3 position;  // Cube vertices (-1 to 1)

// Outputs to fragment shader
layout(location = 0) out vec3 texCoord;  // Cubemap sample direction

// Camera uniforms
layout(set = 0, binding = 0) uniform Camera {
    mat4 viewMatrix;
    mat4 projectionMatrix;
    vec3 cameraPosition;
} u_camera;

void main() {
    // Remove translation from view matrix (keep only rotation)
    // This makes the skybox appear infinitely far away
    mat4 viewRotation = mat4(mat3(u_camera.viewMatrix));

    // Transform position
    vec4 clipPos = u_camera.projectionMatrix * viewRotation * vec4(position, 1.0);

    // Set depth to 1.0 (maximum depth) to render behind everything
    // Dividing by w ensures depth = 1.0 after perspective division
    gl_Position = clipPos.xyww;

    // Use position as texture coordinate (view direction from camera)
    texCoord = position;
}
