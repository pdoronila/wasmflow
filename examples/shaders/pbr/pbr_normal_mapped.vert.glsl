#version 450

// Vertex shader for PBR with normal mapping

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 tangent;

layout(set = 0, binding = 0) uniform CameraUniforms {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
};

layout(location = 0) out vec3 frag_world_pos;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;
layout(location = 3) out vec3 frag_view_dir;
layout(location = 4) out vec3 frag_tangent;
layout(location = 5) out vec3 frag_bitangent;

void main() {
    vec4 world_pos = vec4(position, 1.0);
    frag_world_pos = world_pos.xyz;
    frag_normal = normalize(normal);
    frag_uv = uv;
    frag_tangent = normalize(tangent);

    // Calculate bitangent (assuming right-handed coordinate system)
    // Note: Some models store the tangent.w component to flip bitangent if needed
    frag_bitangent = normalize(cross(frag_normal, frag_tangent));

    // Calculate view direction
    frag_view_dir = normalize(camera_position - world_pos.xyz);

    gl_Position = projection * view * world_pos;
}
