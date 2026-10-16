// PlaatCraft Wii
#include <gccore.h>
#include <malloc.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wiiuse/wpad.h>

#include "blocks_data.h"
#include "camera.h"
#include "canvas.h"
#include "chunk.h"
#include "cursor.h"
#include "pages.h"
#include "perlin.h"
#include "random.h"
#include "utils.h"
#include "world.h"

// GX state
#define DEFAULT_FIFO_SIZE (256 * 1024)
GXRModeObj *screenmode;

// Triangle display list
// clang-format off
_Alignas(32) uint8_t triangleList[32] = {
    GX_TRIANGLES | GX_VTXFMT0,
    0, 3,           // number of vertexes (16-bit big endian)
    0, 1,           // position
    255, 0, 0, 255, // color
    -1, -1,         // position
    0, 255, 0, 255, // color
    1, -1,          // position
    0, 0, 255, 255  // color
};
// clang-format on

// Game state
bool running = true;
Camera camera = {{0, CHUNK_SIZE / 2, 0}, 0, 0};
World world;
float triangleRotation = 0;

// Init
void init(void) {
    // Init canvas
    canvas_init();

    // Init cursor
    cursor_init();

    // Load block textures
    block_textures_load();

    // Init perlin noise
    random_seed = 1234;
    perlin_init(random_seed);

    // Init world
    world_init(&world, &camera);
}

// Update
void update(void) {
    // Read Wii remote
    cursor_update();
    uint32_t buttonsHeld = WPAD_ButtonsDown(0);
    if (buttonsHeld & WPAD_BUTTON_HOME) {
        running = false;
    }
    if (buttonsHeld & WPAD_BUTTON_UP) {
        if (page.focus <= 0) {
            page.focus = page.maxFocus - 1;
        } else {
            page.focus--;
        }
    }
    if (buttonsHeld & WPAD_BUTTON_DOWN) {
        if (page.focus >= page.maxFocus - 1) {
            page.focus = 0;
        } else {
            page.focus++;
        }
    }
    triangleRotation += 1;

    // Update camera for controls
    if (page.type == PAGE_GAME) {
        camera_update(&camera);
    }
}

// Render
void render(void) {
    // Set clear color
    if (page.type == PAGE_GAME) {
        GX_SetCopyClear((GXColor){176, 232, 252, 255}, GX_MAX_Z24);
    } else {
        GX_SetCopyClear((GXColor){128, 128, 128, 255}, GX_MAX_Z24);
    }

    // ### Draw 3D scene ###
    GX_SetZMode(GX_ENABLE, GX_LEQUAL, GX_TRUE);

    // Set projection matrix
    Mtx44 perspectiveMatrix;
    guPerspective(perspectiveMatrix, 45, (float)screenmode->viWidth / (float)screenmode->viHeight, 0.1, 1000);
    GX_LoadProjectionMtx(perspectiveMatrix, GX_PERSPECTIVE);

    if (page.type == PAGE_GAME) {
        GX_SetCullMode(GX_CULL_BACK);

        // Render world
        world_render(&world);

        GX_SetCullMode(GX_CULL_NONE);
    } else {
        GX_SetCullMode(GX_CULL_NONE);

        // Set vertex pipeline
        GX_ClearVtxDesc();
        GX_SetVtxDesc(GX_VA_POS, GX_DIRECT);
        GX_SetVtxDesc(GX_VA_CLR0, GX_DIRECT);
        GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_S8, 0);
        GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
        GX_SetNumChans(1);
        GX_SetNumTexGens(0);
        GX_SetTevOrder(GX_TEVSTAGE0, GX_TEXCOORDNULL, GX_TEXMAP_NULL, GX_COLOR0A0);
        GX_SetTevOp(GX_TEVSTAGE0, GX_PASSCLR);

        // Draw triangles
        for (int32_t y = -2; y < 2; y++) {
            for (int32_t x = -2; x < 2; x++) {
                // Set triangle matrix
                Mtx triangleMatrix;
                guMtxRotDeg(triangleMatrix, 'x', triangleRotation);
                Mtx tempMatrix;
                guMtxRotDeg(tempMatrix, 'y', triangleRotation);
                guMtxConcat(triangleMatrix, tempMatrix, triangleMatrix);
                guMtxTransApply(triangleMatrix, triangleMatrix, x * 2 + 1, y * 2 + 1, -10);
                GX_LoadPosMtxImm(triangleMatrix, GX_PNMTX0);

                // Draw triangle display list
                GX_CallDispList(triangleList, sizeof(triangleList));
            }
        }
    }

    // ### Draw HUD ###
    canvas_begin(screenmode->viWidth, screenmode->viHeight);

    // Render current page
    if (page.type == PAGE_MENU) {
        pages_menu_render();
    } else if (page.type == PAGE_MAPS) {
        pages_maps_render();
    } else if (page.type == PAGE_HELP) {
        pages_help_render();
    } else if (page.type == PAGE_SETTINGS) {
        pages_settings_render();
    }

    char debugBuffer[255];
    sprintf(debugBuffer, "%dx%d %dx%d", screenmode->fbWidth, screenmode->xfbHeight, screenmode->viWidth,
            screenmode->viHeight);
    canvas_fill_text(8, 8, debugBuffer, 8, 0xffffffff);

    if (page.type == PAGE_GAME) {
        // Draw crosshair
        canvas_fill_rect((screenmode->viWidth - 24) / 2, (screenmode->viHeight - 4) / 2, 24, 4, 0xffffffff);
        canvas_fill_rect((screenmode->viWidth - 4) / 2, (screenmode->viHeight - 24) / 2, 4, 24, 0xffffffff);

        // Draw more debug info
        int32_t cameraChunkX = round(world.camera->position.x / CHUNK_SIZE);
        int32_t cameraChunkY = round(world.camera->position.y / CHUNK_SIZE);
        int32_t cameraChunkZ = round(world.camera->position.z / CHUNK_SIZE);
        sprintf(debugBuffer, "Pos: %.3gx%.3gx%.3g - %dx%dx%d", world.camera->position.x, world.camera->position.y,
                world.camera->position.z, cameraChunkX, cameraChunkY, cameraChunkZ);
        canvas_fill_text(8, 8 + 8 + 4, debugBuffer, 8, 0xffffffff);
    }

    // Render cursor
    if (page.type != PAGE_GAME) {
        cursor_render();
    }

    canvas_end();
}

// Main
void poweroff(void) { running = false; }

void wpad_poweroff(int32_t chan) {
    if (chan == WPAD_CHAN_ALL) {
        running = false;
    }
}

int main(void) {
    // Init video
    VIDEO_Init();
    VIDEO_SetBlack(true);

    // Get screen mode
    screenmode = VIDEO_GetPreferredMode(NULL);
    VIDEO_Configure(screenmode);
    if (CONF_GetAspectRatio() == CONF_ASPECT_16_9) {
        screenmode->viWidth = (float)screenmode->viHeight * (16.f / 9.f);
    }

    // Alloc two framebuffers to toggle between
    void *frameBuffers[] = {MEM_K0_TO_K1(SYS_AllocateFramebuffer(screenmode)),
                            MEM_K0_TO_K1(SYS_AllocateFramebuffer(screenmode))};
    uint32_t fb_index = 0;
    VIDEO_SetNextFramebuffer(frameBuffers[fb_index]);

    // Wait for next frame
    VIDEO_Flush();
    VIDEO_WaitVSync();
    if (screenmode->viTVMode & VI_NON_INTERLACE) VIDEO_WaitVSync();

    // Init gx fifo buffer
    uint8_t *gx_fifo = MEM_K0_TO_K1(memalign(32, DEFAULT_FIFO_SIZE));
    memset(gx_fifo, 0, DEFAULT_FIFO_SIZE);
    GX_Init(gx_fifo, DEFAULT_FIFO_SIZE);

    // Init other gx stuff
    GX_SetViewport(0, 0, screenmode->fbWidth, screenmode->efbHeight, 0, 1);
    float yscale = GX_GetYScaleFactor(screenmode->efbHeight, screenmode->xfbHeight);
    uint32_t xfbHeight = GX_SetDispCopyYScale(yscale);
    GX_SetDispCopySrc(0, 0, screenmode->fbWidth, screenmode->efbHeight);
    GX_SetDispCopyDst(screenmode->fbWidth, xfbHeight);
    GX_SetCopyFilter(screenmode->aa, screenmode->sample_pattern, GX_TRUE, screenmode->vfilter);
    GX_SetFieldMode(screenmode->field_rendering,
                    screenmode->viHeight == 2 * screenmode->xfbHeight ? GX_ENABLE : GX_DISABLE);
    GX_SetDispCopyGamma(GX_GM_1_0);

    GX_InvVtxCache();
    GX_InvalidateTexAll();
    VIDEO_SetBlack(false);

    // Init wpad buttons
    WPAD_Init();
    WPAD_SetDataFormat(WPAD_CHAN_ALL, WPAD_FMT_BTNS_ACC_IR);
    WPAD_SetVRes(0, screenmode->viWidth, screenmode->viHeight);

    // Set power off handlers
    SYS_SetPowerCallback(poweroff);
    WPAD_SetPowerButtonCallback(wpad_poweroff);

    // Game loop
    init();
    while (running) {
        // Update
        update();

        // Render
        render();

        // Present framebuffer
        GX_DrawDone();
        fb_index ^= 1;
        GX_CopyDisp(frameBuffers[fb_index], GX_TRUE);
        VIDEO_SetNextFramebuffer(frameBuffers[fb_index]);

        // Wait for next frame
        VIDEO_Flush();
        VIDEO_WaitVSync();
        if (screenmode->viTVMode & VI_NON_INTERLACE) VIDEO_WaitVSync();
    }

    // Power off console
    SYS_ResetSystem(SYS_POWEROFF, 0, 0);
    return 0;
}
