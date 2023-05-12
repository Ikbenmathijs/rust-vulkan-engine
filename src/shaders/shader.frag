#version 450

layout(location=0) out vec4 outColor;

layout(location=0) in vec3 fragColor;
layout(location=1) in vec2 texCoord;
layout(location=2) in vec3 normal;
layout(location=3) in vec3 pos;

layout(binding=1) uniform sampler2D texSampler;

layout(push_constant) uniform PushConstants {
    layout(offset=64) float opacity;
    
} pcs;

void main() {
    float ambientStrength = 0.1;
    vec4 texureColor = vec4(texture(texSampler, texCoord).rgb, pcs.opacity);

    vec4 ambient = texureColor * ambientStrength;


}