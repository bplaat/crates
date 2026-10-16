#pragma once

#include <gccore.h>
#include <stdbool.h>
#include <stdint.h>

typedef enum MaterialLayer {
    MaterialLayerBrickRed,
    MaterialLayerCactusSide,
    MaterialLayerCactusTop,
    MaterialLayerDirt,
    MaterialLayerDirtGrass,
    MaterialLayerGrassTop,
    MaterialLayerGreystone,
    MaterialLayerLava,
    MaterialLayerLeaves,
    MaterialLayerSand,
    MaterialLayerStone,
    MaterialLayerStoneCoal,
    MaterialLayerStoneDiamond,
    MaterialLayerStoneGold,
    MaterialLayerStoneIron,
    MaterialLayerTrunkSide,
    MaterialLayerTrunkTop,
    MaterialLayerWater,
    MaterialLayerWood,
    MaterialLayerCount,
} MaterialLayer;

_Alignas(32) static const uint8_t materials_tpl[] = {
#embed "../target/materials.tpl"
};

static bool materials_load(TPLFile* tpl, GXTexObj textures[MaterialLayerCount]) {
    if (TPL_OpenTPLFromMemory(tpl, (void*)materials_tpl, sizeof(materials_tpl)) != 1)
        return false;
    for (int material = 0; material < MaterialLayerCount; material++) {
        if (TPL_GetTexture(tpl, material, &textures[material]) != 0)
            return false;
    }
    return true;
}
