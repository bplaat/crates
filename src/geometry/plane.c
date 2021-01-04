// PlaatCraft - Plane Geometry

#include "geometry/plane.h"
#include <stdlib.h>

float PLANE_VERTICES[] = {
    // Vertex position, Texture position
    -0.5, -0.5,  0, 0,
     0.5, -0.5,  1, 0,
     0.5,  0.5,  1, 1,
     0.5,  0.5,  1, 1,
    -0.5,  0.5,  0, 1,
    -0.5, -0.5,  0, 0
};

Plane* plane_new() {
    Plane* plane = malloc(sizeof(Plane));

    glGenVertexArrays(1, &plane->vertex_array);
    glBindVertexArray(plane->vertex_array);

    glGenBuffers(1, &plane->vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, plane->vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(PLANE_VERTICES), PLANE_VERTICES, GL_STATIC_DRAW);

    return plane;
}

void plane_use(Plane* plane) {
    glBindVertexArray(plane->vertex_array);
}

void plane_free(Plane* plane) {
    glDeleteVertexArrays(1, &plane->vertex_array);
    glDeleteBuffers(1, &plane->vertex_buffer);
    free(plane);
}
