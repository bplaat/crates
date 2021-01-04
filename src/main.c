#include <stdlib.h>
#include "log.h"
#include "game.h"

int main() {
    log_init();
    game_init();

    Game* game = game_new("PlaatCraft", 1280, 720);
    game_start(game);
    game_free(game);

    return EXIT_SUCCESS;
}
