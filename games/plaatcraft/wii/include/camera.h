#pragma once

#include <gccore.h>

typedef struct Camera {
    guVector position;
    float pitch;
    float yaw;
} Camera;

void camera_update(Camera *camera);
