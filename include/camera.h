// PlaatCraft - Camera Header

#ifndef CAMERA_H
#define CAMERA_H

#include "math/vector4.h"
#include "math/matrix4.h"

#undef near
#undef far

typedef struct Camera {
    float fov;
    float aspect;
    float near;
    float far;

    Vector4 position;
    Vector4 rotation;

    Matrix4 projectionMatrix;
    Matrix4 viewMatrix;
} Camera;

Camera* camera_new(float fov, float aspect, float near, float far);

void camera_update_matrix(Camera* camera);

void camera_free(Camera* camera);

#endif
