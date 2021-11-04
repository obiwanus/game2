#version 450 core

layout(std140, binding = 1) uniform UTransforms {
    mat4 mvp;
    mat4 proj;
    mat4 view;
    mat4 model;
    mat4 sun_vp;
}
uTransforms;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;  // ignored for now
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec2 outUV;

uniform mat4 model;

void main() {
    gl_Position = uTransforms.mvp * model * vec4(inPosition, 1.0);
    outUV = inUV;
}
