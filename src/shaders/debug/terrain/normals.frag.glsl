#version 450 core

in vec3 fColor;

out vec4 Color;

void main() { Color = vec4(fColor, 1.0); }
