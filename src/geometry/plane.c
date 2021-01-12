// PlaatCraft - Plane Geometry

#include "geometry/plane.h"
#include <stdlib.h>

float PLANE_VERTICES[] = {
    // Vertex position, Texture position
     0.5f, -0.5f,  1.f, 0.f,
     0.5f,  0.5f,  1.f, 1.f,
    -0.5f,  0.5f,  0.f, 1.f,

    -0.5f,  0.5f,  0.f, 1.f,
    -0.5f, -0.5f,  0.f, 0.f,
     0.5f, -0.5f,  1.f, 0.f
};

Plane* plane_new() {
    Plane* plane = malloc(sizeof(Plane));

    glGenVertexArrays(1, &plane->vertex_array);
    plane_enable(plane);

    glGenBuffers(1, &plane->vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, plane->vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(PLANE_VERTICES), PLANE_VERTICES, GL_STATIC_DRAW);

    plane_disable(plane);

    return plane;
}

void plane_enable(Plane* plane) {
    glBindVertexArray(plane->vertex_array);
}

void plane_disable(Plane* plane) {
    (void)plane;
    glBindVertexArray(0);
}

void plane_free(Plane* plane) {
    glDeleteVertexArrays(1, &plane->vertex_array);
    glDeleteBuffers(1, &plane->vertex_buffer);
    free(plane);
}
