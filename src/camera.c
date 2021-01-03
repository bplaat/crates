// PlaatCraft - Camera

#include "camera.h"
#include <stdlib.h>

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

    return camera;
}

void camera_update_matrix(Camera* camera) {
    matrix4_perspective(&camera->projectionMatrix, camera->fov, camera->aspect, camera->near, camera->far);

    matrix4_rotate_x(&camera->viewMatrix, camera->rotation.x);

    Matrix4 temp_matrix;
    matrix4_rotate_y(&temp_matrix, camera->rotation.y);
    matrix4_mul(&camera->viewMatrix, &temp_matrix);

    matrix4_rotate_z(&temp_matrix, camera->rotation.z);
    matrix4_mul(&camera->viewMatrix, &temp_matrix);

    Vector4 temp_vector = { -camera->position.x, -camera->position.y, camera->position.z, 1 };
    matrix4_translate(&temp_matrix, &temp_vector);
    matrix4_mul(&camera->viewMatrix, &temp_matrix);
}

void camera_free(Camera* camera) {
    free(camera);
}
