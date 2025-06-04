/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

import { EventEmitter } from 'events';
import { webusb } from 'usb';
import log from '../log.js';

export default class UsbDmx extends EventEmitter {
    async connect() {
        this.device = await webusb.requestDevice({ filters: [{ vendorId: 0x0403, productId: 0x6001 }] });
        if (!this.device) {
            log.error(`Can't connect to the USB DMX512 connector`);
        }
        await this.device.open();
        log.info(`Connected to the USB DMX512 connector`);

        await this.device.claimInterface(0);

        // Set baud rate to 250000
        await this.device.controlTransferOut({
            requestType: 'vendor',
            recipient: 'device',
            request: 3,
            value: 12,
            index: 0,
        });
    }

    async _send() {
        this.emit('dmxRequest');

        // Toggle line break
        await this.device.controlTransferOut({
            requestType: 'vendor',
            recipient: 'device',
            request: 4,
            value: 8 | (2 << 11) | (0 << 8) | (1 << 14),
            index: 0,
        });
        await this.device.controlTransferOut({
            requestType: 'vendor',
            recipient: 'device',
            request: 4,
            value: 8 | (2 << 11) | (0 << 8) | (0 << 14),
            index: 0,
        });

        await this.device.transferOut(0x02, this.dmx);
        setTimeout(() => this._send(), 22.4);
    }

    startSending() {
        this._send();
    }
}
