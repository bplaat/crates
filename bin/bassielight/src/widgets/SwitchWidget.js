/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

import Widget from './Widget.js';

export default class SwitchWidget extends Widget {
    constructor(fixtures, x, y) {
        super();
        this.fixtures = fixtures;
        this.x = x;
        this.y = y;
        this.width = 8;
        this.height = 1;
        this.toggle = new Array(4).fill(false);
        this.pressed = new Array(4).fill(false);
        this.on('buttonPress', ({ x, y }) => this._onButtonPress(x, y));
        this.on('buttonRelease', ({ x, y }) => this._onButtonRelease(x, y));
    }

    _onButtonPress(x) {
        const state = this.fixtures[0].state;
        for (let i = 0; i < 4; i++) {
            if (x === i * 2) {
                this.toggle[i] = !state[i];
                for (const fixture of this.fixtures) {
                    fixture.state[i] = this.toggle[i];
                }
            }
            if (x === i * 2 + 1) {
                this.pressed[i] = true;
                for (const fixture of this.fixtures) {
                    fixture.state[i] = true;
                }
            }
        }
    }

    _onButtonRelease(x) {
        for (let i = 0; i < 4; i++) {
            if (x === i * 2 + 1) {
                this.toggle[i] = false;
                this.pressed[i] = false;
                for (const fixture of this.fixtures) {
                    fixture.state[i] = false;
                }
            }
        }
    }

    draw(launchpad) {
        const labels = this.fixtures[0].labels || [];
        for (let i = 0; i < 4; i++) {
            launchpad.write(
                this.x + i * 2,
                this.y,
                this.toggle[i] ? 2 : 1,
                labels[i] !== null ? `${labels[i]}\nToggle` : null
            );
            launchpad.write(
                this.x + i * 2 + 1,
                this.y,
                this.pressed[i] ? 2 : 1,
                labels[i] !== null ? `${labels[i]}\nPush` : null
            );
        }
    }
}
