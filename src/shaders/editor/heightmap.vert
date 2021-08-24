#version 450 core

// In normalised device coordinates
const vec2 VERTICES[] = vec2[](vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));

out VS_OUT { vec2 uv; }
vs_out;

void main() {
    vec2 vertex = VERTICES[gl_VertexID];

    // Texture coords
    vs_out.uv = 0.5 + (vertex / 2.0);

    gl_Position = vec4(vertex, 0.0, 1.0);
}
