#version 450

// Vertex shader for multiple lights

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(set = 0, binding = 0) uniform CameraUniforms {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
};

layout(location = 0) out vec3 frag_world_pos;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;
layout(location = 3) out vec3 frag_view_dir;

void main() {
    vec4 world_pos = vec4(position, 1.0);
    frag_world_pos = world_pos.xyz;
    frag_normal = normal;
    frag_uv = uv;

    // Calculate view direction
    frag_view_dir = normalize(camera_position - world_pos.xyz);

    gl_Position = projection * view * world_pos;
}
