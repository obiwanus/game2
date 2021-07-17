#version 330 core

in VS_OUTPUT {
    //   vec3 normal;
    vec3 frag_pos;
    vec3 color;
}
IN;

uniform vec3 cursor;

out vec4 Color;

void main() {
    //   vec3 view_direction = normalize(-IN.frag_pos);

    float distance_to_cursor = distance(IN.frag_pos.xz, cursor.xz);
    if (distance_to_cursor < 3.0) {
        Color = vec4(IN.color, 1.0);
    } else {
        Color = vec4(IN.color * 0.5, 1.0);
    }
}
