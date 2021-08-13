#version 450 core
out vec4 FragColor;

uniform float time;

void main() {
    FragColor =
        vec4(1.0 * (0.25 * cos(time * 2.5) + 0.75), 1.0 * (0.5 * sin(time * 5.0) + 0.5), 0.0, 1.0);
}
