#version 450 core
layout(location = 0) in vec3 Position;

out vec3 TexCoords;

layout(std140, binding = 1) uniform UTransforms {
    mat4 mvp;
    mat4 proj;
    mat4 view;
    mat4 model;
}
uTransforms;

void main() {
    TexCoords = Position;
    mat4 skybox_view = mat4(mat3(uTransforms.view));  // remove the translation component
    vec4 pos = uTransforms.proj * skybox_view * vec4(Position, 1.0);
    gl_Position = pos.xyww;
}
