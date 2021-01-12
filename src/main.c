// PlaatCraft - Main

#include <stdio.h>
#include <stdlib.h>
#include "log.h"
#include "config.h"
#include "game.h"

#include <unistd.h>

int main(void) {
    // Init log
    log_init();

    // Create game title string
    char game_title[128];
    sprintf(game_title, "PlaatCraft v%d.%d", PLAATCRAFT_VERSION_MAJOR, PLAATCRAFT_VERSION_MINOR);
    log_info("%s", game_title);

    // Init game object
    game_init();

    // Create game object
    Game* game = game_new(game_title, 1280, 720);
    game_start(game);
    game_free(game);

    return EXIT_SUCCESS;
}
