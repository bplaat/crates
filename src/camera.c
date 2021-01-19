// PlaatCraft - Camera

#include "camera.h"
#include <stdlib.h>
#include <GLFW/glfw3.h>
#include "config.h"

Camera* camera_new(float fov, float aspect, float near, float far) {
    Camera* camera = malloc(sizeof(Camera));

    camera->fov = fov;
    camera->aspect = aspect;
    camera->near = near;
    camera->far = far;

    camera->position.x = 0;
    camera->position.y = 0;
    camera->position.z = 0;

    camera->rotation.x = 0;
    camera->rotation.y = 0;
    camera->rotation.z = 0;

    camera->velocity.x = 0;
    camera->velocity.y = 0;
    camera->velocity.z = 0;
    camera->speed = CAMERA_SPEED;
    camera->is_moving_forward = false;
    camera->is_moving_left = false;
    camera->is_moving_right = false;
    camera->is_moving_backward = false;
    camera->is_moving_up = false;
    camera->is_moving_down = false;

    camera->is_first_mouse_movement = false;
    camera->yaw = 0;
    camera->pitch = 0;

    return camera;
}

void camera_key_callback(Camera* camera, int key, int scancode, int action, int mods) {
    (void)scancode;
    (void)mods;

    if (action == GLFW_PRESS) {
        if (key == GLFW_KEY_W || key == GLFW_KEY_UP) {
            camera->is_moving_forward = true;
        }
        if (key == GLFW_KEY_S || key == GLFW_KEY_DOWN) {
            camera->is_moving_backward = true;
        }
        if (key == GLFW_KEY_A || key == GLFW_KEY_LEFT) {
            camera->is_moving_left = true;
        }
        if (key == GLFW_KEY_D || key == GLFW_KEY_RIGHT) {
            camera->is_moving_right = true;
        }
        if (key == GLFW_KEY_SPACE) {
            camera->is_moving_up = true;
        }
        if (key == GLFW_KEY_LEFT_SHIFT) {
            camera->is_moving_down = true;
        }
    }

    if (action == GLFW_RELEASE) {
        if (key == GLFW_KEY_W || key == GLFW_KEY_UP) {
            camera->is_moving_forward = false;
        }
        if (key == GLFW_KEY_S || key == GLFW_KEY_DOWN) {
            camera->is_moving_backward = false;
        }
        if (key == GLFW_KEY_A || key == GLFW_KEY_LEFT) {
            camera->is_moving_left = false;
        }
        if (key == GLFW_KEY_D || key == GLFW_KEY_RIGHT) {
            camera->is_moving_right = false;
        }
        if (key == GLFW_KEY_SPACE) {
            camera->is_moving_up = false;
        }
        if (key == GLFW_KEY_LEFT_SHIFT) {
            camera->is_moving_down = false;
        }
    }
}

void camera_cursor_position_callback(Camera* camera, double xpos, double ypos) {
    if (camera->is_first_mouse_movement) {
        camera->is_first_mouse_movement = false;
        camera->last_x = xpos;
        camera->last_y = ypos;
    }

    float xoffset = xpos - camera->last_x;
    float yoffset = camera->last_y - ypos;
    camera->last_x = xpos;
    camera->last_y = ypos;

    float sensitivity = 0.002;

    camera->yaw -= xoffset * sensitivity;

    camera->pitch += yoffset * sensitivity;
    if (camera->pitch > radians(90)) camera->pitch = radians(90);
    if (camera->pitch < -radians(90)) camera->pitch = -radians(90);

    camera->rotation.x = -camera->pitch;
    camera->rotation.y = -camera->yaw;
}

void camera_update(Camera* camera, float delta) {
    camera->velocity.z -= camera->velocity.z * (camera->speed * 3) * delta;
    camera->velocity.x -= camera->velocity.x * (camera->speed * 3) * delta;
    camera->velocity.y -= camera->velocity.y * (camera->speed * 3) * delta;

    if (camera->is_moving_forward) {
        camera->velocity.z += camera->speed * delta;;
    }
    if (camera->is_moving_backward) {
        camera->velocity.z -= camera->speed * delta;;
    }

    if (camera->is_moving_left) {
        camera->velocity.x -= camera->speed * delta;;
    }
    if (camera->is_moving_right) {
        camera->velocity.x += camera->speed * delta;;
    }

    if (camera->is_moving_up) {
        camera->velocity.y += camera->speed * delta;;
    }
    if (camera->is_moving_down) {
        camera->velocity.y -= camera->speed * delta;;
    }

    Matrix4 rotation_matrix;
    matrix4_rotate_y(&rotation_matrix, camera->rotation.y);

    Vector4 update = camera->velocity;
    vector4_mul(&update, &rotation_matrix);
    vector4_add(&camera->position, &update);

    camera_update_matrix(camera);
}

void camera_update_matrix(Camera* camera) {
    matrix4_perspective(&camera->projection_matrix, camera->fov, camera->aspect, camera->near, camera->far);

    matrix4_rotate_x(&camera->view_matrix, camera->rotation.x);

    Matrix4 temp_matrix;
    matrix4_rotate_y(&temp_matrix, camera->rotation.y);
    matrix4_mul(&camera->view_matrix, &temp_matrix);

    matrix4_rotate_z(&temp_matrix, camera->rotation.z);
    matrix4_mul(&camera->view_matrix, &temp_matrix);

    Vector4 temp_vector = { -camera->position.x, -camera->position.y, camera->position.z, 1 };
    matrix4_translate(&temp_matrix, &temp_vector);
    matrix4_mul(&camera->view_matrix, &temp_matrix);
}

void camera_free(Camera* camera) {
    free(camera);
}
