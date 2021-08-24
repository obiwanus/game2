#version 450 core

in VS_OUT { vec2 uv; }
fs_in;

uniform vec2 cursor;       // normalised [0:1]
uniform float brush_size;  // normalised [0:1]
uniform float delta_time;

layout(binding = 0) uniform sampler2D brush_texture;

layout(location = 0) out vec4 Color;

void main() {
    // Note that brush_size is actually more like brush radius (i.e. half brush real size)
    vec2 brush_uv = vec2(0.5, 0.5) + (fs_in.uv - cursor) / brush_size;
    vec3 brush_value = texture(brush_texture, brush_uv).rrr * delta_time;  // TODO: sensitivity

    // Will be blended with what's currently in the heightmap
    Color = vec4(brush_value, 1.0);
}
