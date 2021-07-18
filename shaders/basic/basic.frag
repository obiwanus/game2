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

    float distance_to_cursor = clamp(distance(IN.frag_pos.xz, cursor.xz) / 5, 0.2, 1.0);

    if (0.87 < distance_to_cursor && distance_to_cursor < 0.9) {
        Color = mix(vec4(1.0, 1.0, 0.5, 1.0), vec4(IN.color, 1.0), distance_to_cursor);
    } else {
        Color = mix(vec4(0.8, 0.8, 0.5, 1.0), vec4(IN.color, 1.0), distance_to_cursor);
    }
}
