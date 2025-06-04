/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

import Fixture from './Fixture.js';

export default class P56LED extends Fixture {
    constructor(name, addr) {
        super();
        this.type = 'rgb';
        this.name = name;
        this.addr = addr;

        this.color = { red: 0, green: 0, blue: 0 };
        this.toggleColor = { red: 0, green: 0, blue: 0 };
        this.intensity = 1;

        this.toggleSpeed = 0;
        this.toggleTime = performance.now();
        this.isToggle = false;

        this.strobeSpeed = 0;
        this.strobeTime = performance.now();
        this.isStrobe = false;
    }

    toggle() {
        this.isToggle = !this.isToggle;
    }

    strobe() {
        this.isStrobe = !this.isStrobe;
    }

    automatic(dmx) {
        const CHANNEL_MODE = 5;
        dmx[this.addr + CHANNEL_MODE] = 255;
    }

    update(dmx) {
        const CHANNEL_RED = 0;
        const CHANNEL_GREEN = 1;
        const CHANNEL_BLUE = 2;

        if (performance.now() - this.toggleTime >= this.toggleSpeed) {
            this.toggleTime = performance.now();
            if (this.toggleSpeed !== 0) this.toggle();
        }
        if (performance.now() - this.strobeTime >= this.strobeSpeed) {
            this.strobeTime = performance.now();
            if (this.strobeSpeed !== 0) this.strobe();
        }

        if ((this.color.red === 0 && this.color.green === 0 && this.color.blue === 0) || this.isStrobe) {
            dmx[this.addr + CHANNEL_RED] = 0;
            dmx[this.addr + CHANNEL_GREEN] = 0;
            dmx[this.addr + CHANNEL_BLUE] = 0;
            return;
        }

        if (this.isToggle) {
            dmx[this.addr + CHANNEL_RED] = this.toggleColor.red * this.intensity;
            dmx[this.addr + CHANNEL_GREEN] = this.toggleColor.green * this.intensity;
            dmx[this.addr + CHANNEL_BLUE] = this.toggleColor.blue * this.intensity;
        } else {
            dmx[this.addr + CHANNEL_RED] = this.color.red * this.intensity;
            dmx[this.addr + CHANNEL_GREEN] = this.color.green * this.intensity;
            dmx[this.addr + CHANNEL_BLUE] = this.color.blue * this.intensity;
        }
    }
}
