/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import './dipswitch.css';

/// DIP switch view of a DMX address, switch 1 is the least significant bit
export function DipSwitch({
    value,
    bits = 9,
    onChange,
}: {
    value: number;
    bits?: number;
    onChange?: (value: number) => void;
}) {
    return (
        <div class="dipswitch" title={`DMX address ${value}`}>
            {Array.from({ length: bits }, (_, bit) => {
                const isOn = (value & (1 << bit)) !== 0;
                return (
                    <button
                        key={bit}
                        type="button"
                        class={`dipswitch-switch ${isOn ? 'is-on' : ''}`}
                        onClick={() => onChange?.(value ^ (1 << bit))}
                    >
                        <span class="dipswitch-slot">
                            <span class="dipswitch-knob" />
                        </span>
                        <span class="dipswitch-label">{bit + 1}</span>
                    </button>
                );
            })}
        </div>
    );
}
