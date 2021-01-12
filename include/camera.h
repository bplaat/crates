// PlaatCraft - Camera Header

#ifndef CAMERA_H
#define CAMERA_H

#include <stdbool.h>
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
    Matrix4 cameraMatrix;

    Vector4 velocity;
    bool is_moving_forward;
    bool is_moving_backward;
    bool is_moving_left;
    bool is_moving_right;
    bool is_moving_up;
    bool is_moving_down;

    bool is_first_mouse_movement;
    double last_x;
    double last_y;
    float yaw;
    float pitch;
} Camera;

Camera* camera_new(float fov, float aspect, float near, float far);

void camera_key_callback(Camera* camera, int key, int scancode, int action, int mods);

void camera_cursor_position_callback(Camera* camera, double xpos, double ypos);

void camera_update(Camera* camera, float delta);

void camera_update_matrix(Camera* camera);

void camera_free(Camera* camera);

#endif
