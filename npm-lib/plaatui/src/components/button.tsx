/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './button.css';
import { cx } from '../utils.ts';

export function Button({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('button', 'is-primary', extraClass)} />;
}

export function SecondaryButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('button', 'is-secondary', extraClass)} />;
}

export function DangerButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('button', 'is-danger', extraClass)} />;
}

export function IconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('button', 'is-icon', extraClass)} />;
}

export function SmallIconButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('button', 'is-icon', 'is-small', extraClass)} />;
}

export function TextButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('text-button', extraClass)} />;
}

export function DangerTextButton({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('text-button', 'is-danger', extraClass)} />;
}
