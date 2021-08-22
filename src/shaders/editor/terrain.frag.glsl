#version 450 core

in TES_OUT {
    vec2 tile_uv;
    vec3 frag_pos;
}
fs_in;

out vec4 Color;

uniform vec2 cursor;
uniform float brush_size;

layout(binding = 0) uniform sampler2D terrain_texture;
layout(binding = 2) uniform sampler2D brush_texture;

void main() {
    vec2 patch_uv = fs_in.tile_uv * 64.0;
    vec4 terrain_color = texture(terrain_texture, patch_uv);

    vec2 brush_uv = vec2(0.5, 0.5) + (fs_in.frag_pos.xz - cursor) / brush_size;
    const vec4 brush_color = vec4(0.75, 0.45, 0.92, 1.0);
    const vec4 brush_highlight = vec4(0.69, 0.67, 0.91, 1.0);
    float brush_value = sqrt(texture(brush_texture, brush_uv).r);

    vec4 base_color = mix(terrain_color, brush_color, brush_value);

    float t = smoothstep(0.1, 0.11, brush_value) - smoothstep(0.11, 0.12, brush_value);
    Color = mix(base_color, brush_highlight, 0.5 * t);
}
