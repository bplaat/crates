/*
 * Copyright (c) 2014 Gabriele Cirulli
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

function LocalStorageManager() {
    this.bestScoreKey = 'bestScore';
    this.gameStateKey = 'gameState';
}

// Helper functions for cookies
LocalStorageManager.prototype._getCookie = function (name) {
    const value = document.cookie.match('(^|;)\\s*' + name + '\\s*=\\s*([^;]+)');
    return value ? decodeURIComponent(value.pop()) : null;
};

LocalStorageManager.prototype._setCookie = function (name, value, days = 365) {
    const expires = new Date(Date.now() + days * 864e5).toUTCString();
    document.cookie = name + '=' + encodeURIComponent(value) + '; expires=' + expires + '; path=/';
};

LocalStorageManager.prototype._removeCookie = function (name) {
    document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
};

// Best score getters/setters
LocalStorageManager.prototype.getBestScore = function () {
    return this._getCookie(this.bestScoreKey) || 0;
};

LocalStorageManager.prototype.setBestScore = function (score) {
    this._setCookie(this.bestScoreKey, score);
};

// Game state getters/setters and clearing
LocalStorageManager.prototype.getGameState = function () {
    var stateJSON = this._getCookie(this.gameStateKey);
    return stateJSON ? JSON.parse(stateJSON) : null;
};

LocalStorageManager.prototype.setGameState = function (gameState) {
    this._setCookie(this.gameStateKey, JSON.stringify(gameState));
};

LocalStorageManager.prototype.clearGameState = function () {
    this._removeCookie(this.gameStateKey);
};
