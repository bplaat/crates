/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './table.css';
import { cx } from '../utils.ts';

export function Table({ class: extraClass, ...props }: JSX.IntrinsicElements['table']) {
    return <table {...props} class={cx('table', extraClass)} />;
}
