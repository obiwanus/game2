#version 450 core

in TES_OUT {
    vec4 frag_pos_sun_space;
    vec3 frag_pos;
    vec2 tile_uv;
}
fs_in;

out vec4 Color;

uniform vec2 cursor;
uniform float brush_size;

layout(binding = 0) uniform sampler2D terrain_texture;
layout(binding = 2) uniform sampler2D brush_texture;
layout(binding = 3) uniform sampler2D shadow_map;

float calc_shadow(vec4 frag_pos) {
    vec3 proj_coords = frag_pos.xyz / frag_pos.w;
    proj_coords = proj_coords * 0.5 + 0.5;
    float closest_depth = texture(shadow_map, proj_coords.xy).r;
    float frag_depth = proj_coords.z;
    float bias = 0.005;
    // TODO: adjust bias based on the angle
    return (frag_depth - bias) > closest_depth ? 1.0 : 0.0;
}

void main() {
    vec2 patch_uv = fs_in.tile_uv * 64.0;
    vec4 terrain_color = texture(terrain_texture, patch_uv);

    vec2 brush_uv = vec2(0.5, 0.5) + (fs_in.frag_pos.xz - cursor) / brush_size;
    const vec4 brush_color = vec4(0.75, 0.45, 0.92, 1.0);
    const vec3 brush_highlight = vec3(0.69, 0.67, 0.91);
    float brush_value = texture(brush_texture, brush_uv).r;

    vec3 base_color = mix(terrain_color, brush_color, brush_value).xyz;
    vec3 ambient = 0.35 * base_color;

    float t = smoothstep(0.1, 0.11, brush_value) - smoothstep(0.11, 0.12, brush_value);
    vec3 color = mix(base_color, brush_highlight, 0.5 * t);

    float shadow = calc_shadow(fs_in.frag_pos_sun_space);
    Color = vec4(ambient + (1.0 - shadow) * (color * 0.65), 1.0);
}
