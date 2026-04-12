/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';

const BTN_PRIMARY_CLASS =
    'px-4 py-2 bg-yellow-400 hover:bg-yellow-500 dark:bg-yellow-900/40 dark:hover:bg-yellow-900/60 dark:text-yellow-400 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer';

const BTN_SECONDARY_CLASS =
    'px-4 py-2 rounded-lg text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-zinc-600 disabled:opacity-60 transition-colors cursor-pointer';

const BTN_DANGER_CLASS =
    'px-4 py-2 bg-red-500 hover:bg-red-600 dark:bg-red-900/50 dark:hover:bg-red-900/70 dark:text-red-300 disabled:opacity-60 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer flex items-center gap-2';

const BTN_ICON_CLASS = 'p-2 rounded-full hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

const BTN_SMALL_ICON_CLASS =
    'p-1.5 rounded-lg text-gray-400 dark:text-gray-500 hover:bg-gray-100 dark:hover:bg-zinc-700 transition-colors cursor-pointer';

export function Button({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_PRIMARY_CLASS} ${extraClass}` : BTN_PRIMARY_CLASS} />;
}

export function SecondaryButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_SECONDARY_CLASS} ${extraClass}` : BTN_SECONDARY_CLASS} />;
}

export function DangerButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_DANGER_CLASS} ${extraClass}` : BTN_DANGER_CLASS} />;
}

export function IconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_ICON_CLASS} ${extraClass}` : BTN_ICON_CLASS} />;
}

export function SmallIconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={extraClass ? `${BTN_SMALL_ICON_CLASS} ${extraClass}` : BTN_SMALL_ICON_CLASS} />;
}
