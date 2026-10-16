#include "canvas.h"


#include "font_chars.h"
#include "font_texture.h"
#include "font_texture_tpl.h"

#define FONT_RENDER_SIZE 8

static GXTexObj fontTexture;

void canvas_init(void) {
    // Load font texture
    TPLFile fontTPL;
    TPL_OpenTPLFromMemory(&fontTPL, (void *)font_texture_tpl, font_texture_tpl_size);
    TPL_GetTexture(&fontTPL, font, &fontTexture);
    GX_InitTexObjFilterMode(&fontTexture, GX_NEAR, GX_NEAR);
}

void canvas_begin(uint32_t screenWidth, uint32_t screenHeight) {
    GX_SetZMode(GX_DISABLE, GX_LEQUAL, GX_TRUE);
    GX_SetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);

    // Set ortographic matrix
    Mtx44 projectionMatrix;
    guOrtho(projectionMatrix, 0, screenHeight, 0, screenWidth, -1, 1);
    GX_LoadProjectionMtx(projectionMatrix, GX_ORTHOGRAPHIC);

    // Set vertex pipeline
    GX_ClearVtxDesc();
    GX_SetVtxDesc(GX_VA_POS, GX_DIRECT);
    GX_SetVtxDesc(GX_VA_CLR0, GX_DIRECT);
    GX_SetVtxDesc(GX_VA_TEX0, GX_DIRECT);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
    GX_SetNumChans(1);
    GX_SetNumTexGens(1);
    GX_SetTexCoordGen(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY);
    GX_SetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GX_SetTevOp(GX_TEVSTAGE0, GX_MODULATE);
}

void canvas_end(void) { GX_SetBlendMode(GX_BM_NONE, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR); }

void canvas_fill_rect(float x, float y, float width, float height, uint32_t color) {
    // Load font texture
    GX_LoadTexObj(&fontTexture, GX_TEXMAP0);

    // clang-format off
    Mtx matrix = {
        {width, 0, 0, x + width / 2},
        {0, height, 0, y + height / 2},
        {0, 0, 1, 0}
    };
    // clang-format on
    GX_LoadPosMtxImm(matrix, GX_PNMTX0);

    float left = 4 / 128.f; // Coords to first white pixel
    float top = 1 / 64.f;
    float right = (4 + 1) / 128.f;
    float bottom = (1 + 1) / 64.f;
    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
    GX_Position2f32(-0.5, -0.5);
    GX_Color1u32(color);
    GX_TexCoord2f32(left, top);
    GX_Position2f32(0.5, -0.5);
    GX_Color1u32(color);
    GX_TexCoord2f32(right, top);
    GX_Position2f32(0.5, 0.5);
    GX_Color1u32(color);
    GX_TexCoord2f32(right, bottom);
    GX_Position2f32(-0.5, 0.5);
    GX_Color1u32(color);
    GX_TexCoord2f32(left, bottom);
    GX_End();
}

void canvas_draw_image(GXTexObj *texture, float x, float y, float width, float height) {
    GX_LoadTexObj(texture, GX_TEXMAP0);

    // clang-format off
    Mtx matrix = {
        {width, 0, 0, x + width / 2},
        {0, height, 0, y + height / 2},
        {0, 0, 1, 0}
    };
    // clang-format on
    GX_LoadPosMtxImm(matrix, GX_PNMTX0);

    GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
    GX_Position2f32(-0.5, -0.5);
    GX_Color1u32(0xffffffff);
    GX_TexCoord2f32(0, 0);
    GX_Position2f32(0.5, -0.5);
    GX_Color1u32(0xffffffff);
    GX_TexCoord2f32(1, 0);
    GX_Position2f32(0.5, 0.5);
    GX_Color1u32(0xffffffff);
    GX_TexCoord2f32(1, 1);
    GX_Position2f32(-0.5, 0.5);
    GX_Color1u32(0xffffffff);
    GX_TexCoord2f32(0, 1);
    GX_End();
}

void canvas_fill_text(float x, float y, char *text, float textSize, uint32_t textColor) {
    // Load font texture
    GX_LoadTexObj(&fontTexture, GX_TEXMAP0);

    float scale = textSize / FONT_RENDER_SIZE;
    char *c = text;
    while (*c != '\0') {
        FontChar *font_char = &font_chars[*c - 32];

        float width = font_char->width * scale;
        float height = font_char->height * scale;
        // clang-format off
        Mtx letterMatrix = {
            {width, 0, 0, x + width / 2},
            {0, height, 0, y + height / 2}, //  - font_char->originY
            {0, 0, 1, 0}
        };
        // clang-format on
        GX_LoadPosMtxImm(letterMatrix, GX_PNMTX0);

        // Draw letter
        float left = font_char->x / 128.f;
        float top = font_char->y / 64.f;
        float right = (font_char->x + font_char->width) / 128.f;
        float bottom = (font_char->y + font_char->height) / 64.f;
        GX_Begin(GX_QUADS, GX_VTXFMT0, 4);
        GX_Position2f32(-0.5, -0.5);
        GX_Color1u32(textColor);
        GX_TexCoord2f32(left, top);
        GX_Position2f32(0.5, -0.5);
        GX_Color1u32(textColor);
        GX_TexCoord2f32(right, top);
        GX_Position2f32(0.5, 0.5);
        GX_Color1u32(textColor);
        GX_TexCoord2f32(right, bottom);
        GX_Position2f32(-0.5, 0.5);
        GX_Color1u32(textColor);
        GX_TexCoord2f32(left, bottom);
        GX_End();

        x += font_char->width * scale;
        c++;
    }
}
