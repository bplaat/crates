#include "blocks_data.h"

#include "blocks_texture.h"
#include "blocks_texture_tpl.h"

BlockTexture block_textures[] = {
    {.type = grass, .name = "Grass"},
    {.type = dirt_grass, .name = "Dirt Grass"},
    {.type = dirt, .name = "Dirt"},
    {.type = sand, .name = "Sand"},
    {.type = stone, .name = "Stone"},
    {.type = stone_coal0, .name = "Stone Coal 0"},
    {.type = stone_coal1, .name = "Stone Coal 1"},
    {.type = stone_gold0, .name = "Stone Gold 0"},
    {.type = stone_gold1, .name = "Stone Gold 1"},
    {.type = stone_diamond0, .name = "Stone Diamond 0"},
    {.type = stone_diamond1, .name = "Stone Diamond 1"},
    {.type = water, .name = "Water"},
    {.type = trunk_oak_top, .name = "Trunk Oak Top"},
    {.type = trunk_oak_side, .name = "Trunk Oak Side"},
    {.type = trunk_beech_top, .name = "Trunk Beech Top"},
    {.type = trunk_beech_side, .name = "Trunk Beech Side"},
    {.type = wood_oak, .name = "Wood Oak"},
    {.type = wood_beech, .name = "Wood Beech"},
    {.type = leaves_green, .name = "Leaves Green"},
    {.type = leaves_orange, .name = "Leaves Orange"},
    {.type = cactus_top, .name = "Cactus Top"},
    {.type = cactus_side, .name = "Cactus Side"},
};

Block blocks[] = {
    {.type = BLOCK_AIR, .name = "Air"},
    {.type = BLOCK_GRASS, .name = "Grass", .textures = {grass, dirt_grass, dirt_grass, dirt_grass, dirt_grass, dirt}},
    {.type = BLOCK_DIRT, .name = "Dirt", .textures = {dirt, dirt, dirt, dirt, dirt, dirt}},
    {.type = BLOCK_SAND_TOP, .name = "Sand", .textures = {sand, sand, sand, sand, sand, sand}},
    {.type = BLOCK_SAND, .name = "Sand", .textures = {sand, sand, sand, sand, sand, sand}},
    {.type = BLOCK_STONE, .name = "Stone", .textures = {stone, stone, stone, stone, stone, stone}},
    {.type = BLOCK_STONE_COAL0,
     .name = "Stone Coal 0",
     .textures = {stone_coal0, stone_coal0, stone_coal0, stone_coal0, stone_coal0, stone_coal0}},
    {.type = BLOCK_STONE_COAL1,
     .name = "Stone Coal 1",
     .textures = {stone_coal1, stone_coal1, stone_coal1, stone_coal1, stone_coal1, stone_coal1}},
    {.type = BLOCK_STONE_GOLD0,
     .name = "Stone Gold 0",
     .textures = {stone_gold0, stone_gold0, stone_gold0, stone_gold0, stone_gold0, stone_gold0}},
    {.type = BLOCK_STONE_GOLD1,
     .name = "Stone Gold 1",
     .textures = {stone_gold1, stone_gold1, stone_gold1, stone_gold1, stone_gold1, stone_gold1}},
    {.type = BLOCK_STONE_DIAMOND0,
     .name = "Stone Diamond 0",
     .textures = {stone_diamond0, stone_diamond0, stone_diamond0, stone_diamond0, stone_diamond0, stone_diamond0}},
    {.type = BLOCK_STONE_DIAMOND1,
     .name = "Stone Diamond 1",
     .textures = {stone_diamond1, stone_diamond1, stone_diamond1, stone_diamond1, stone_diamond1, stone_diamond1}},
    {.type = BLOCK_WATER, .name = "Water", .textures = {water, water, water, water, water, water}},
    {.type = BLOCK_TRUNK_OAK,
     .name = "Trunk Oak",
     .textures = {trunk_oak_top, trunk_oak_side, trunk_oak_side, trunk_oak_side, trunk_oak_side, trunk_oak_top}},
    {.type = BLOCK_TRUNK_BEECH,
     .name = "Trunk Beech",
     .textures = {trunk_beech_top, trunk_beech_side, trunk_beech_side, trunk_beech_side, trunk_beech_side,
                  trunk_beech_top}},
    {.type = BLOCK_WOOD_OAK,
     .name = "Wood Oak",
     .textures = {wood_oak, wood_oak, wood_oak, wood_oak, wood_oak, wood_oak}},
    {.type = BLOCK_WOOD_BEECH,
     .name = "Wood Beech",
     .textures = {wood_beech, wood_beech, wood_beech, wood_beech, wood_beech, wood_beech}},
    {.type = BLOCK_LEAVES_GREEN,
     .name = "Leaves Green",
     .textures = {leaves_green, leaves_green, leaves_green, leaves_green, leaves_green, leaves_green}},
    {.type = BLOCK_LEAVES_ORANGE,
     .name = "Leaves Orange",
     .textures = {leaves_orange, leaves_orange, leaves_orange, leaves_orange, leaves_orange, leaves_orange}},
    {.type = BLOCK_CACTUS,
     .name = "Cactus",
     .textures = {cactus_top, cactus_side, cactus_side, cactus_side, cactus_side, cactus_top}},
};

void block_textures_load(void) {
    TPLFile blocksTPL;
    TPL_OpenTPLFromMemory(&blocksTPL, (void *)blocks_texture_tpl, blocks_texture_tpl_size);
    for (size_t i = 0; i < sizeof(block_textures) / sizeof(BlockTexture); i++) {
        BlockTexture *blockTexture = &block_textures[i];
        TPL_GetTexture(&blocksTPL, blockTexture->type, &blockTexture->texture);
    }
}
