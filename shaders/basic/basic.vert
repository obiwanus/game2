#version 330 core

layout(location = 0) in vec3 Position;
layout(location = 1) in vec3 Normal;
layout(location = 2) in vec2 TexCoord;

uniform mat4 proj;
uniform mat4 view;
// uniform mat4 model;

out VS_OUTPUT {
    vec3 normal;
    vec3 frag_pos;
    vec3 frag_pos_view;
    vec3 color;
    vec2 tex_coord;
}
OUT;

void main() {
    //   gl_Position = proj * view * model * vec4(Position, 1.0);
    gl_Position = proj * view * vec4(Position, 1.0);
    //   OUT.normal = mat3(transpose(inverse(view * model))) * Normal;  // @performance: don't
    //   inverse
    OUT.normal = mat3(transpose(inverse(view))) * Normal;  // @performance: don't inverse
    //   OUT.frag_pos = (view * model * vec4(Position, 1.0)).xyz;
    OUT.frag_pos_view = (view * vec4(Position, 1.0)).xyz;
    OUT.frag_pos = Position;
    //   OUT.color = Color.xyz * vec3(0.8, 0.8, 0.8);
    OUT.color = vec3(0.4, 0.5, 0.2);
    OUT.tex_coord = TexCoord;
}
