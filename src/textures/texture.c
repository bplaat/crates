// PlaatCraft - Texture

#include "textures/texture.h"
#include <stdlib.h>
#include "stb_image/stb_image.h"
#include "log.h"

Texture* texture_new_from_file(char* path, bool is_transparent) {
    int32_t image_width, image_height, image_channels;
    uint8_t* image_bitmap = stbi_load(path, &image_width, &image_height, &image_channels, is_transparent ? STBI_rgb_alpha : STBI_rgb);
    if (!image_bitmap) {
        log_error("Can't load image %s", path);
    }

    Texture* texture = texture_new_from_bitmap(path, image_width, image_height, is_transparent, image_bitmap);

    stbi_image_free(image_bitmap);

    return texture;
}

Texture* texture_new_from_bitmap(char* path, int width, int height, bool is_transparent, uint8_t* bitmap) {
    Texture* texture = malloc(sizeof(Texture));
    texture->path = path;
    texture->width = width;
    texture->height = height;
    texture->is_transparent = is_transparent;

    glGenTextures(1, &texture->texture);
    texture_enable(texture);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexImage2D(GL_TEXTURE_2D, 0, is_transparent ? GL_RGBA : GL_RGB, width, height, 0, is_transparent ? GL_RGBA : GL_RGB, GL_UNSIGNED_BYTE, bitmap);
    glGenerateMipmap(GL_TEXTURE_2D);

    texture_disable(texture);

    return texture;
}

void texture_enable(Texture* texture) {
    if (texture->is_transparent) {
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    glBindTexture(GL_TEXTURE_2D, texture->texture);
}

void texture_disable(Texture* texture) {
    glBindTexture(GL_TEXTURE_2D, 0);

    if (texture->is_transparent) {
        glDisable(GL_BLEND);
    }
}

void texture_free(Texture* texture) {
    glDeleteTextures(1, &texture->texture);
    free(texture);
}
