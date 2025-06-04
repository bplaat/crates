/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

import Fixture from './Fixture.js';

export default class MultiDimMKII extends Fixture {
    constructor(name, addr, options = {}) {
        super();
        this.type = 'multidim';
        this.name = name;
        this.addr = addr;
        this.state = new Array(4).fill(false);
        if ('labels' in options) {
            this.labels = options.labels;
        }
    }

    update(dmx) {
        dmx[this.addr + 0] = this.state[0] ? 255 : 0;
        dmx[this.addr + 1] = this.state[1] ? 255 : 0;
        dmx[this.addr + 2] = this.state[2] ? 255 : 0;
        dmx[this.addr + 3] = this.state[3] ? 255 : 0;
    }
}
