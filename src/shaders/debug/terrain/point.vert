#version 450 core
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

out vec3 Color;

layout(std140, binding = 1) uniform UTransforms {
    mat4 mvp;
    mat4 proj;
    mat4 view;
    mat4 model;
}
uTransforms;

void main() {
    gl_Position = uTransforms.mvp * vec4(inPosition, 1.0);
    Color = inColor;
}
