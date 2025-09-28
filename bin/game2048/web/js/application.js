/*
 * Copyright (c) 2014 Gabriele Cirulli
 *
 * SPDX-License-Identifier: MIT
 */

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(function () {
    new GameManager(4, KeyboardInputManager, HTMLActuator, LocalStorageManager);
});

window.addEventListener('contextmenu', (event) => event.preventDefault());
