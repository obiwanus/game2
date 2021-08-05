#version 410 core

layout(quads, fractional_odd_spacing) in;

uniform sampler2D heightmap;

uniform mat4 mvp;

in TCS_OUT { vec2 uv; }
tes_in[];

out TES_OUT { vec2 uv; }
tes_out;

void main() {
    vec2 uv1 = mix(tes_in[0].uv, tes_in[1].uv, gl_TessCoord.x);
    vec2 uv2 = mix(tes_in[2].uv, tes_in[3].uv, gl_TessCoord.x);
    vec2 uv = mix(uv2, uv1, gl_TessCoord.y);

    vec4 p1 = mix(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_TessCoord.x);
    vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);
    vec4 p = mix(p2, p1, gl_TessCoord.y);

    // TODO: add displacement
    p.y += texture(heightmap, uv).r * 150.0;
    gl_Position = mvp * p;
    tes_out.uv = uv;
}
