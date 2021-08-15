#version 450 core
in vec3 inColor;
out vec4 FragColor;

void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
// void main() { FragColor = vec4(inColor, 1.0); }
