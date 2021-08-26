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

out TES_OUT {
    vec4 frag_pos_sun_space;
    vec3 frag_pos;
    vec3 normal;
    vec2 tile_uv;
}
tes_out;

float sample_height(vec2 uv) { return texture(heightmap, uv).r * terrain_max_height; }

vec3 calc_normal(vec2 uv) {
    // @speed: maybe pass texture size in the uniform
    // or maybe build a normal map while drawing on heightmap
    vec2 heightmap_size = textureSize(heightmap, 0);
    vec2 texel_size = 1.0 / heightmap_size;
    float L = sample_height(uv - vec2(texel_size.x, 0));
    float R = sample_height(uv + vec2(texel_size.x, 0));
    float T = sample_height(uv - vec2(0, texel_size.y));
    float B = sample_height(uv + vec2(0, texel_size.y));

    // WARNING: hardcoded 1024
    vec2 texel_size_world = 1024.0 / heightmap_size;
    vec3 horizontal = vec3(2.0 * texel_size_world.x, R - L, 0.0);
    vec3 vertical = vec3(0.0, B - T, 2.0 * texel_size_world.y);

    vec3 normal = normalize(cross(vertical, horizontal));
    return normal;
}

void main() {
    vec2 uv1 = mix(tes_in[0].tile_uv, tes_in[1].tile_uv, gl_TessCoord.x);
    vec2 uv2 = mix(tes_in[2].tile_uv, tes_in[3].tile_uv, gl_TessCoord.x);
    vec2 tile_uv = mix(uv2, uv1, gl_TessCoord.y);

    vec4 p1 = mix(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_TessCoord.x);
    vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);
    vec4 p = mix(p2, p1, gl_TessCoord.y);

    p.y = sample_height(tile_uv);
    gl_Position = uTransforms.mvp * p;
    tes_out.tile_uv = tile_uv;
    tes_out.frag_pos = p.xyz;
    tes_out.frag_pos_sun_space = uTransforms.sun_vp * vec4(tes_out.frag_pos, 1.0);
    tes_out.normal = calc_normal(tile_uv);
}
