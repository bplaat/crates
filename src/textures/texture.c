// PlaatCraft - Texture

#include "textures/texture.h"
#include <stdlib.h>
#include "textures/stb_image.h"
#include "log.h"

Texture* texture_new(char* path, bool isTransparent) {
    Texture* texture = malloc(sizeof(texture));
    texture->path = path;
    texture->isTransparent = isTransparent;

    int32_t image_channels;
    uint8_t* image_buffer = stbi_load(path, &texture->width, &texture->height, &image_channels, isTransparent ? STBI_rgb_alpha : STBI_rgb);
    if (!image_buffer) {
        log_error("Can't load image: %s\n", path);
    }

    glGenTextures(1, &texture->texture);
    glBindTexture(GL_TEXTURE, texture->texture);

    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MIN_FILTER, GL_NEAREST_MIPMAP_NEAREST);
    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MAG_FILTER, GL_NEAREST);

    glTexImage2D(GL_TEXTURE_2D, 0, isTransparent ? GL_RGBA : GL_RGB, texture->width, texture->height, 0, isTransparent ? GL_RGBA : GL_RGB, GL_UNSIGNED_BYTE, image_buffer);

    stbi_image_free(image_buffer);

    glGenerateMipmap(GL_TEXTURE_2D_ARRAY);

    return texture;
}

void texture_free(Texture* texture) {
    glDeleteTextures(1, &texture->texture);
}
