#version 450 core
out vec4 FragColor;

in vec3 TexCoords;

uniform sampler1D SkyTexture;

void main() { FragColor = texture(SkyTexture, 0.5 + TexCoords.y / 2.0); }
