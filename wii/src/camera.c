#include "camera.h"

#include <gccore.h>
#include <stdlib.h>
#include <wiiuse/wpad.h>

#include "utils.h"

void camera_update(Camera *camera) {
    const float cameraSpeed = 0.5;
    const float cameraSensitivity = 0.0003;

    // Buttons camera movement
    guVector update = {0};
    WPAD_ScanPads();
    if (WPAD_ButtonsDown(0) & WPAD_BUTTON_HOME) {
        exit(0);
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_UP) {
        update.z = -cameraSpeed;
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_LEFT) {
        update.x = -cameraSpeed;
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_RIGHT) {
        update.x = cameraSpeed;
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_DOWN) {
        update.z = cameraSpeed;
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_A) {
        update.y = cameraSpeed;
    } else if (WPAD_ButtonsHeld(0) & WPAD_BUTTON_B) {
        update.y = -cameraSpeed;
    }
    if (update.x != 0 || update.y != 0 || update.z != 0) {
        Mtx rotationMatrix;
        guMtxRotRad(rotationMatrix, 'y', camera->yaw);
        guVecMultiply(rotationMatrix, &update, &update);
        guVecAdd(&camera->position, &update, &camera->position);
    }

    // Numchunk camera rotation movement
    struct expansion_t data;
    WPAD_Expansion(WPAD_CHAN_0, &data);
    if (data.type == WPAD_EXP_NUNCHUK) {
        int32_t xoffset = data.nunchuk.js.pos.x - data.nunchuk.js.center.x;
        int32_t yoffset = data.nunchuk.js.pos.y - data.nunchuk.js.center.y;
        camera->yaw -= (float)xoffset * cameraSensitivity;
        camera->pitch += (float)yoffset * cameraSensitivity;
        if (camera->pitch < -radians(90)) camera->pitch = -radians(90);
        if (camera->pitch > radians(90)) camera->pitch = radians(90);
    }
}
