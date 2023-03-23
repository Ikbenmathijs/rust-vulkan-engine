#version 450



layout(binding=0) uniform UBO {
    mat4 model;
    mat4 proj;
    mat4 view;
} ubo;


layout(location=0) out vec3 fragColor;

layout(location=0) in vec2 inPos;
layout(location=1) in vec3 inColor;



void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPos, 0.0, 1.0);
    fragColor = inColor;
}