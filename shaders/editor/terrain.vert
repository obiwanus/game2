#version 330 core

layout(location = 0) in vec3 in_Pos;
layout(location = 1) in vec3 in_Normal;
layout(location = 2) in vec2 in_TexCoord;

uniform mat4 proj;
uniform mat4 view;

uniform vec2 terrain_origin;
uniform float terrain_size;
uniform sampler2D heightmap;

const mat4 model = mat4(1.0);
const float MAX_HEIGHT = 30.0;

out VS_OUTPUT {
    vec3 normal;
    vec3 frag_pos;
    vec3 frag_pos_view;
    vec3 color;  // @note: not using now, consider using as an additional way to paint terrain
    vec2 tex_coord;
}
OUT;

void main() {
    // Read about sampling textures in GLSL
    vec2 uv = (in_Pos.xz - terrain_origin) / terrain_size;
    float height = texture(heightmap, uv).r * MAX_HEIGHT;
    vec4 position = vec4(in_Pos.x, height, in_Pos.z, 1.0);
    vec4 view_pos = view * model * position;
    gl_Position = proj * view_pos;
    OUT.normal = mat3(transpose(inverse(view * model))) * in_Normal;  // @performance: don't inverse
    OUT.frag_pos_view = view_pos.xyz;
    OUT.frag_pos = position.xyz;
    OUT.color = vec3(0.4, 0.5, 0.2);
    OUT.tex_coord = in_TexCoord;
}
