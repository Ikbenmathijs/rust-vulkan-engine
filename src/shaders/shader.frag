#version 450

layout(location=0) out vec4 outColor;

layout(location=0) in vec3 fragColor;
layout(location=1) in vec2 texCoord;

layout(binding=1) uniform sampler2D texSampler;

layout(push_constant) uniform PushConstants {
    layout(offset=64) float opacity;
} pcs;

void main() {
    //outColor = vec4(fragColor, 1);
    outColor = vec4(texture(texSampler, texCoord).rgb, pcs.opacity);
}