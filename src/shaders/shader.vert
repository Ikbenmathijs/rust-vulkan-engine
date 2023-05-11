#version 450



layout(binding=0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;


layout(location=0) out vec3 fragColor;
layout(location=1) out vec2 fragTexCoord;

layout(location=0) in vec3 inPos;
layout(location=1) in vec3 inColor;
layout(location=2) in vec2 texCoord;
layout(location=3) in vec3 normal;



void main() {
    gl_Position = ubo.proj * ubo.view * pcs.model * vec4(inPos, 1.0);
    fragColor = normal;
    fragTexCoord = texCoord;
}