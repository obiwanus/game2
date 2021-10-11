#version 450 core
layout(triangles) in;
layout(line_strip, max_vertices = 6) out;

in TES_OUT {
    vec4 frag_pos_sun_space;
    vec3 frag_pos;
    vec3 normal;
    vec2 tile_uv;
}
gs_in[];

out vec3 fColor;

const float MAGNITUDE = 2.0;

layout(std140, binding = 1) uniform UTransforms {
    mat4 mvp;
    mat4 proj;
    mat4 view;
    mat4 model;
    mat4 sun_vp;
}
uTransforms;

void GenerateLine(int index) {
    vec4 vertex_pos = vec4(gs_in[index].frag_pos, 1.0);

    gl_Position = uTransforms.mvp * vertex_pos;
    fColor = vec3(1.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = uTransforms.mvp * (vertex_pos + vec4(gs_in[index].normal, 0.0) * MAGNITUDE);
    fColor = vec3(1.0, 1.0, 0.0);
    EmitVertex();

    EndPrimitive();
}

void main() {
    GenerateLine(0);  // first vertex normal
    GenerateLine(1);  // second vertex normal
    GenerateLine(2);  // third vertex normal
}
