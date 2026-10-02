/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

export function ButtonList<T extends string | number | null>({
    label,
    value,
    options,
    onChange,
}: {
    label: string;
    value: T;
    options: { value: T; label: string }[];
    onChange: (value: T) => void;
}) {
    return (
        <div class="list" role="group" aria-label={label}>
            {options.map((option) => (
                <button
                    key={String(option.value)}
                    class={`button ${option.value === value ? 'is-selected' : ''}`}
                    aria-pressed={option.value === value}
                    onClick={() => onChange(option.value)}
                >
                    {option.label}
                </button>
            ))}
        </div>
    );
}
