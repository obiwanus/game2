#version 450 core
layout(location = 0) in vec3 vPosition;

out vec3 TexCoords;
out vec3 Color;

uniform mat4 proj;
uniform mat4 view;

void main() {
    TexCoords = vPosition;
    mat4 skybox_view = mat4(mat3(view));  // remove the translation component
    vec4 pos = proj * skybox_view * vec4(vPosition, 1.0);
    gl_Position = pos.xyww;
}
