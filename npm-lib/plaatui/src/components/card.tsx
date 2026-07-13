/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import { cx } from '../utils.ts';
import './card.css';

export type CardProps = JSX.IntrinsicElements['div'] & { padded?: boolean; clipped?: boolean };

export function Card({ class: extraClass, padded = true, clipped, ...props }: CardProps) {
    return <div {...props} class={cx('card', padded && 'is-padded', clipped && 'is-clipped', extraClass)} />;
}

export type CardTitleProps = JSX.IntrinsicElements['h2'] & { tight?: boolean };

export function CardTitle({ class: extraClass, tight, ...props }: CardTitleProps) {
    return <h2 {...props} class={cx('card-title', tight && 'is-tight', extraClass)} />;
}
