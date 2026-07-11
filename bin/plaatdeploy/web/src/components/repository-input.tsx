/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 * SPDX-License-Identifier: MIT
 */

interface Props {
    id: string;
    repositories: string[];
    value: string;
    disabled?: boolean;
    placeholder?: string;
    onChange: (repository: string) => void;
}

export function RepositoryInput({ id, repositories, value, disabled, placeholder, onChange }: Props) {
    return (
        <>
            <input
                id={id}
                class="input"
                list={`${id}-options`}
                value={value}
                onInput={(event) => onChange((event.target as HTMLInputElement).value)}
                disabled={disabled}
                placeholder={placeholder}
                required
            />
            <datalist id={`${id}-options`}>
                {repositories.map((repository) => (
                    <option key={repository} value={repository} />
                ))}
            </datalist>
        </>
    );
}
