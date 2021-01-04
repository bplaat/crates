// PlaatCraft - Plane Geometry Header

#ifndef PLANE_H
#define PLANE_H

#include "glad/glad.h"

extern float PLANE_VERTICES[];

typedef struct Plane {
    GLuint vertex_array;
    GLuint vertex_buffer;
} Plane;

Plane* plane_new();

void plane_enable(Plane* plane);

void plane_disable(Plane* plane);

void plane_free(Plane* plane);

#endif
