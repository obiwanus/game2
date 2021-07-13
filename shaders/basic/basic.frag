#version 330 core

in VS_OUTPUT {
//   vec3 normal;
  vec3 frag_pos;
  vec3 color;
}
IN;

out vec4 Color;

void main() {
//   vec3 view_direction = normalize(-IN.frag_pos);

  Color = vec4(IN.color, 1.0);
}
