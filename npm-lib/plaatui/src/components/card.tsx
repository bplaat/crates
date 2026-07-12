/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import './card.css';
import { cx } from '../utils.ts';

export type CardProps = JSX.IntrinsicElements['div'] & { padded?: boolean };

export function Card({ class: extraClass, padded = true, ...props }: CardProps) {
    return <div {...props} class={cx('card', padded && 'is-padded', extraClass)} />;
}
