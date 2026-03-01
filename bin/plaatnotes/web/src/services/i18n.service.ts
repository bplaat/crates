/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { signal } from '@preact/signals';
import en from '../i18n/en.json';
import nl from '../i18n/nl.json';

const translations: Record<string, Record<string, string>> = { en, nl };

function detectLanguage(): string {
    const lang = navigator.language ?? 'en';
    if (lang.startsWith('nl')) return 'nl';
    return 'en';
}

export const $language = signal<string>(detectLanguage());

export function setLanguage(lang: string) {
    $language.value = lang in translations ? lang : 'en';
}

export function t(key: string, ...args: string[]): string {
    const dict = translations[$language.value] ?? translations['en'];
    const value = dict[key] ?? translations['en'][key] ?? key;
    return value.replace(/\{(\d+)\}/g, (_, i) => args[parseInt(i)] ?? '');
}

const DATE_FORMAT: Intl.DateTimeFormatOptions = {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
};

export function formatDate(date: string | Date | undefined | null): string {
    if (!date) return '—';
    const d = new Date(date);
    if (isNaN(d.getTime())) return '—';
    return d.toLocaleString($language.value, DATE_FORMAT);
}
