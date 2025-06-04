/*
 * Copyright (c) 2023, Bastiaan van der Plaat <bastiaan.v.d.plaat@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

function rand(min, max) {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

const openPage = document.getElementById('open-page');
const boardPage = document.getElementById('board-page');
const boardContainer = document.getElementById('board-container');
let boardButtons = [];

const MessageType = {
    BOARD_COLORS: 0x00,
    BOARD_LABELS: 0x01,
    BUTTON_PRESS: 0x10,
    BUTTON_RELEASE: 0x11,
};

let ws;
function connect() {
    ws = new WebSocket(`ws://${location.host}/ws`);
    ws.binaryType = 'arraybuffer';
    ws.onopen = () => {
        openPage.classList.add('is-hidden');
        boardPage.classList.remove('is-hidden');
    };
    ws.onmessage = (event) => {
        let pos = 0;
        const view = new DataView(event.data);
        const type = view.getUint8(pos++);

        if (type === MessageType.BOARD_COLORS) {
            for (let y = -1; y < 8; y++) {
                for (let x = 0; x < 9; x++) {
                    const color = view.getUint8(pos++);
                    if (!boardButtons[y + 1][x]) continue;
                    boardButtons[y + 1][x].classList.remove('is-black');
                    boardButtons[y + 1][x].classList.remove('is-yellow');
                    boardButtons[y + 1][x].classList.remove('is-red');
                    if (color === 0) boardButtons[y + 1][x].classList.add('is-black');
                    if (color === 1) boardButtons[y + 1][x].classList.add('is-yellow');
                    if (color === 2) boardButtons[y + 1][x].classList.add('is-red');
                }
            }
        }

        if (type === MessageType.BOARD_LABELS) {
            const decoder = new TextDecoder();
            for (let y = -1; y < 8; y++) {
                for (let x = 0; x < 9; x++) {
                    const labelLength = view.getUint16(pos, true); pos += 2;
                    const label = decoder.decode(new Uint8Array(event.data, pos, labelLength)); pos += labelLength;
                    if (!boardButtons[y + 1][x]) continue;
                    boardButtons[y + 1][x].textContent = label;
                }
            }
        }
    };
}

const LETTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
function createBoard() {
    const row = document.createElement('div');
    row.className = 'row';
    boardContainer.appendChild(row);

    for (let x = 0; x < 8; x++) {
        const column = document.createElement('div');
        column.className = 'label-horizontal';
        column.textContent = x + 1;
        row.appendChild(column);
    }

    for (let y = -1; y < 8; y++) {
        const row = document.createElement('div');
        row.className = 'row';
        boardContainer.appendChild(row);
        let boardRow = [];
        boardButtons.push(boardRow);

        const width = y === -1 ? 8 : 8 + 1;
        for (let x = 0; x < width; x++) {
            const column = document.createElement('button');
            column.className = `button ${y === -1 || x === 8 ? 'is-round' : ''}`;
            column.setAttribute('data-x', x);
            column.setAttribute('data-y', y);
            column.addEventListener('mousedown', (event) => {
                const message = new ArrayBuffer(1 + 1 + 1);
                const view = new DataView(message);
                let pos = 0;
                view.setUint8(pos++, MessageType.BUTTON_PRESS);
                view.setInt8(pos++, parseInt(event.target.getAttribute('data-x')));
                view.setInt8(pos++, parseInt(event.target.getAttribute('data-y')));
                ws.send(message);
            });
            column.addEventListener('mouseup', (event) => {
                const message = new ArrayBuffer(1 + 1 + 1);
                const view = new DataView(message);
                let pos = 0;
                view.setUint8(pos++, MessageType.BUTTON_RELEASE);
                view.setInt8(pos++, parseInt(event.target.getAttribute('data-x')));
                view.setInt8(pos++, parseInt(event.target.getAttribute('data-y')));
                ws.send(message);
            });
            row.appendChild(column);
            boardRow.push(column);
        }

        if (y !== -1) {
            const column = document.createElement('div');
            column.className = 'label-vertical';
            column.textContent = LETTERS[y];
            row.appendChild(column);
        }
    }
}

createBoard();
setTimeout(connect, rand(500, 750));
