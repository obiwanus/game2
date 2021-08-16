#version 450 core

const vec2 terrain_center = vec2(0.0);
const float PATCH_SIZE = 8.0;  // so that one terrain tile is 1000x1000 units?

const vec2 VERTICES[] = vec2[](vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0));

out VS_OUT { vec2 tile_uv; }
vs_out;

uniform mat4 mvp;

void main() {
    vec2 vertex = VERTICES[gl_VertexID];

    // One terrain tile will always have 64x64 patches
    int x = gl_InstanceID & 63;
    int y = gl_InstanceID >> 6;
    vec2 offset = vec2(x, y);

    // Texture coords
    vs_out.tile_uv = (vertex + offset) / 64.0;

    // Position
    vec2 position = (vertex + vec2(offset.x - 32.0, offset.y - 32.0)) * PATCH_SIZE + terrain_center;

    // TODO: displace height here?
    float height = 0.0;
    gl_Position = vec4(position.x, height, position.y, 1.0);
}
