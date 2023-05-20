#version 450

layout(location=0) out vec4 outColor;

layout(location=0) in vec3 fragColor;
layout(location=1) in vec2 texCoord;
layout(location=2) in vec3 normal;
layout(location=3) in vec3 pos;

layout(binding=1) uniform sampler2D texSampler;

layout(push_constant) uniform PushConstants {
    layout(offset=64) vec3 light_dir;
    layout(offset=76) float opacity;
} pcs;


void main() {

    vec3 N = normalize(normal);

    float diffuse = max(dot(N, pcs.light_dir), 0);

    float ambientStrength = 0.1;
    vec4 textureColor = texture(texSampler, texCoord);

    outColor = vec4(textureColor.rgb  * (diffuse + 0.02), 1);
}