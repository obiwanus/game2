#version 450 core

const vec2 VERTICES[] = vec2[](vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0));

uniform vec2 terrain_center;
uniform int num_patches;
uniform float patch_size;

out VS_OUT { vec2 tile_uv; }
vs_out;

void main() {
    vec2 vertex = VERTICES[gl_VertexID];

    int x = gl_InstanceID % num_patches;
    int y = gl_InstanceID / num_patches;
    vec2 offset = vec2(x, y);

    // Texture coords
    vs_out.tile_uv = (vertex + offset) / float(num_patches);

    // Position
    float half_num = float(num_patches) / 2.0;
    vec2 position = (vertex + vec2(offset.x - half_num, offset.y - half_num)) * patch_size + terrain_center;

    // TODO: displace height here?
    float height = 0.0;
    gl_Position = vec4(position.x, height, position.y, 1.0);
}
