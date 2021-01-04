// PlaatCraft - Plane Geometry Header

#ifndef PLANE_H
#define PLANE_H

#include "glad.h"

extern float PLANE_VERTICES[];

typedef struct Plane {
    GLuint vertex_array;
    GLuint vertex_buffer;
} Plane;

Plane* plane_new();

void plane_use(Plane* plane);

void plane_free(Plane* plane);

#endif
