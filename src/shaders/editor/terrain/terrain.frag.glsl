#version 450 core

in TES_OUT {
    vec4 frag_pos_sun_space;
    vec3 frag_pos;
    vec3 normal;
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
    float frag_depth = proj_coords.z;
    // TODO: adjust bias based on the angle
    float bias = 0.003;
    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcf_depth = texture(shadow_map, proj_coords.xy + vec2(x, y) * texel_size).r;
            shadow += (frag_depth - bias) > pcf_depth ? 1.0 : 0.0;
        }
    }
    return shadow / 9.0;
}

void main() {
    vec2 patch_uv = fs_in.tile_uv * 64.0;
    vec4 terrain_color = texture(terrain_texture, patch_uv);

    vec2 brush_uv = vec2(0.5, 0.5) + (fs_in.frag_pos.xz - cursor) / brush_size;
    const vec4 brush_color = vec4(0.75, 0.45, 0.92, 1.0);
    const vec3 brush_border_color = vec3(0.69, 0.67, 0.91);
    float brush_value = texture(brush_texture, brush_uv).r;
    vec3 base_color = mix(terrain_color, brush_color, brush_value * 1.5).xyz;
    float t = smoothstep(0.1, 0.11, brush_value) - smoothstep(0.11, 0.12, brush_value);
    base_color = mix(base_color, brush_border_color, 0.5 * t);

    vec3 ambient = 0.15 * base_color;
    vec3 normal = normalize(fs_in.normal);
    vec3 light_color = vec3(1.0);
    vec3 light_dir = vec3(0.0, -200.0, -500.0);  // @hardcoded
    float diff = max(dot(light_dir, normal), 0.0);
    vec3 diffuse = diff * light_color;

    // float shadow = calc_shadow(fs_in.frag_pos_sun_space);
    float shadow = 0.0;

    vec3 lighting = (ambient + (1.0 - shadow) * diffuse) * base_color;

    Color = vec4(lighting, 1.0);
}
