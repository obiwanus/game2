#version 330 core

layout(location = 0) in vec3 Position;
layout(location = 1) in vec3 Normal;
layout(location = 2) in vec2 TexCoord;

uniform mat4 proj;
uniform mat4 view;

uniform float heighmap_size;
uniform sampler2D heighmap;

const mat4 model = mat4(1.0);

out VS_OUTPUT {
    vec3 normal;
    vec3 frag_pos;
    vec3 frag_pos_view;
    vec3 color;  // @note: not using now, consider using as an additional way to paint terrain
    vec2 tex_coord;
}
OUT;

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

void main() {
    gl_Position = proj * view * model * vec4(Position, 1.0);
    OUT.normal = mat3(transpose(inverse(view * model))) * Normal;  // @performance: don't inverse
    OUT.frag_pos_view = (view * model * vec4(Position, 1.0)).xyz;
    OUT.frag_pos = Position;
    OUT.color = vec3(0.4, 0.5, 0.2);
    OUT.tex_coord = TexCoord;
}
