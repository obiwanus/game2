#version 330 core

uniform sampler2D u_sampler;
in vec4 v_rgba;
in vec2 v_tc;
out vec4 f_color;

void main() {
    // The texture sampler is sRGB aware
    f_color = v_rgba + abs(texture(u_sampler, v_tc));  // + vec4(1.0, 1.0, v_tc);
}
