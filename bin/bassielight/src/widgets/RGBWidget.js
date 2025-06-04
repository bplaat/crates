/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

import { COLORS, SPEEDS } from '../consts.js';
import Widget from './Widget.js';

export default class RGBWidget extends Widget {
    constructor(fixtures, x, y) {
        super();
        this.fixtures = fixtures;
        this.x = x;
        this.y = y;
        this.width = 8;
        this.height = 7;
        this.pressed = new Array(4).fill(false);
        this.on('buttonPress', ({ x, y }) => this._onButtonPress(x, y));
        this.on('toggle', () => this._onToggle());
        this.on('strobe', () => this._onStrobe());
    }

    _onButtonPress(x, y) {
        // Color
        if (y === 0 || y === 1) {
            if (COLORS[y === 1 ? x + 8 : x]) {
                for (const fixture of this.fixtures) {
                    fixture.color = COLORS[y === 1 ? x + 8 : x];
                }
            }
        }

        // Toggle color
        if (y === 2 || y === 3) {
            if (COLORS[y === 3 ? x + 8 : x]) {
                for (const fixture of this.fixtures) {
                    fixture.toggleColor = COLORS[y === 3 ? x + 8 : x];
                }
            }
        }

        // Intensity
        if (y === 4) {
            for (const fixture of this.fixtures) {
                fixture.intensity = (x + 1) / 8;
            }
        }

        // Toggle speed
        if (y === 5) {
            for (const fixture of this.fixtures) {
                if (SPEEDS[x] === 0) fixture.isToggle = false;
                fixture.toggleSpeed = SPEEDS[x];
            }
        }

        // Strobe speed
        if (y === 6) {
            for (const fixture of this.fixtures) {
                if (SPEEDS[x] === 0) fixture.isStrobe = false;
                fixture.strobeSpeed = SPEEDS[x];
            }
        }
    }

    _onToggle() {
        for (const fixture of this.fixtures) {
            if (fixture.type === 'rgb') {
                fixture.toggle();
            }
        }
    }

    _onStrobe() {
        for (const fixture of this.fixtures) {
            if (fixture.type === 'rgb') {
                fixture.strobe();
            }
        }
    }

    draw(launchpad) {
        // Color
        const color = this.fixtures[0].color;
        for (let i = 0; i < COLORS.length; i++) {
            if (COLORS[i]) {
                launchpad.write(
                    this.x + (i >= 8 ? i - 8 : i),
                    this.y + (i >= 8 ? 1 : 0),
                    color.red === COLORS[i].red && color.green === COLORS[i].green && color.blue === COLORS[i].blue
                        ? 2
                        : 1,
                    `Color\n${COLORS[i].name}`
                );
            }
        }

        // Toggle color
        const toggleColor = this.fixtures[0].toggleColor;
        for (let i = 0; i < COLORS.length; i++) {
            if (COLORS[i]) {
                launchpad.write(
                    this.x + (i >= 8 ? i - 8 : i),
                    this.y + (i >= 8 ? 3 : 2),
                    toggleColor.red === COLORS[i].red && toggleColor.green === COLORS[i].green && toggleColor.blue === COLORS[i].blue
                        ? 2
                        : 1,
                    `Toggle color\n${COLORS[i].name}`
                );
            }
        }

        // Intensity
        const intensity = this.fixtures[0].intensity;
        for (let i = 0; i < SPEEDS.length; i++)
            launchpad.write(
                this.x + i,
                this.y + 4,
                (i + 1) / 8 === intensity ? 2 : 1,
                `Intensity\n${(((i + 1) / 8) * 100).toFixed(0)} %`
            );

        // Toggle speed
        const toggleSpeed = this.fixtures[0].toggleSpeed;
        for (let i = 0; i < SPEEDS.length; i++) {
            launchpad.write(this.x + i, this.y + 5, SPEEDS[i] === toggleSpeed ? 2 : 1, `Toggle speed\n${SPEEDS[i]} ms`);
        }

        // Strobe speed
        const strobeSpeed = this.fixtures[0].strobeSpeed;
        for (let i = 0; i < SPEEDS.length; i++) {
            launchpad.write(this.x + i, this.y + 6, SPEEDS[i] === strobeSpeed ? 2 : 1, `Strobe speed\n${SPEEDS[i]} ms`);
        }
    }
}
