#version 450 core

layout(quads, fractional_odd_spacing) in;

layout(std140, binding = 1) uniform UTransforms {
    mat4 mvp;
    mat4 proj;
    mat4 view;
    mat4 model;
    mat4 sun_vp;
}
uTransforms;

layout(binding = 1) uniform sampler2D heightmap;

uniform float terrain_max_height = 200.0;

in TCS_OUT { vec2 tile_uv; }
tes_in[];

void main() {
    vec2 uv1 = mix(tes_in[0].tile_uv, tes_in[1].tile_uv, gl_TessCoord.x);
    vec2 uv2 = mix(tes_in[2].tile_uv, tes_in[3].tile_uv, gl_TessCoord.x);
    vec2 tile_uv = mix(uv2, uv1, gl_TessCoord.y);

    vec4 p1 = mix(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_TessCoord.x);
    vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);
    vec4 p = mix(p2, p1, gl_TessCoord.y);

    p.y += texture(heightmap, tile_uv).r * terrain_max_height;
    gl_Position = uTransforms.sun_vp * uTransforms.model * p;
}
