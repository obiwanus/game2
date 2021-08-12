#version 450 core

uniform vec3 aabb_min;
uniform vec3 aabb_max;

uniform mat4 mvp;

vec3 VERTICES[] = vec3[](
    aabb_min,
    vec3(aabb_max.x, aabb_min.y, aabb_min.z),
    vec3(aabb_max.x, aabb_max.y, aabb_min.z),
    vec3(aabb_min.x, aabb_max.y, aabb_min.z),

    vec3(aabb_min.x, aabb_min.y, aabb_max.z),
    vec3(aabb_max.x, aabb_min.y, aabb_max.z),
    aabb_max,
    vec3(aabb_min.x, aabb_max.y, aabb_max.z)
);
int INDICES[] = int[](
    0, 1, 2,
    0, 2, 3,
    1, 5, 6,
    1, 6, 2,
    5, 4, 7,
    5, 7, 6,
    4, 0, 3,
    4, 3, 7,
    3, 2, 6,
    3, 6, 7,
    5, 1, 0,
    5, 0, 4
);

// const vec3 aabb_min = vec3(-120.0, 0.0, -120.0);
// const vec3 aabb_max = vec3(120.0, 200.0, 120.0);

// uniform mat4 mvp;

// const vec3 VERTICES[] = vec3[](
//     aabb_min,
//     vec3(aabb_max.x, aabb_min.y, aabb_min.z),
//     vec3(aabb_max.x, aabb_max.y, aabb_min.z),
//     vec3(aabb_min.x, aabb_max.y, aabb_min.z),

//     vec3(aabb_min.x, aabb_min.y, aabb_max.z),
//     vec3(aabb_max.x, aabb_min.y, aabb_max.z),
//     aabb_max,
//     vec3(aabb_min.x, aabb_max.y, aabb_max.z)
// );
// const int INDICES[] = int[](
//     0, 1, 2,
//     0, 2, 3,
//     1, 5, 6,
//     1, 6, 2,
//     5, 4, 7,
//     5, 7, 6,
//     4, 0, 3,
//     4, 3, 7,
//     3, 2, 6,
//     3, 6, 7,
//     5, 1, 0,
//     5, 0, 4
// );

void main() {

    gl_Position = mvp * vec4(VERTICES[INDICES[gl_VertexID]], 1.0);
}
