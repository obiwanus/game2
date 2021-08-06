#version 450 core
out vec4 FragColor;

in vec3 TexCoords;
// in vec3 Color;

uniform sampler2D SkyTexture;
// uniform samplerCube skybox;

void main() {
    // FragColor = texture(skybox, TexCoords);  // * vec4(1.0, 0.7, 0.7, 1.0);
    float y = 0.5 + TexCoords.y / 2.0;
    FragColor = texture(SkyTexture, vec2(0.5, y));
}
