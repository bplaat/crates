/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './fab.css';
import { cx } from '../utils.ts';

export function Fab({ class: extraClass, ...props }: JSX.IntrinsicElements['button']) {
    return <button {...props} class={cx('fab', extraClass)} />;
}
