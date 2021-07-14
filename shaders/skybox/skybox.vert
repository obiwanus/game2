#version 330 core
layout (location = 0) in vec3 Position;

out vec3 TexCoords;

uniform mat4 proj;
uniform mat4 view;

void main()
{
    TexCoords = Position;
    mat4 skybox_view = mat4(mat3(view)); // remove the translation component
    vec4 pos = proj * skybox_view * vec4(Position, 1.0);
    gl_Position = pos.xyww;
}
