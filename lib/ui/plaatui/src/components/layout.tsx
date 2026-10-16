/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type JSX } from 'preact/jsx-runtime';
import { cx } from '../utils.ts';

export type PageProps = JSX.IntrinsicElements['div'] & {
    size?: 'default' | 'narrow' | 'wide';
};

export function Page({ size = 'default', class: extraClass, ...props }: PageProps) {
    return <div {...props} class={cx('page', size !== 'default' && `is-${size}`, extraClass)} />;
}

export function PageTitle({ class: extraClass, ...props }: JSX.IntrinsicElements['h1']) {
    return <h1 {...props} class={cx('page-title', extraClass)} />;
}

export type SectionLabelProps = JSX.IntrinsicElements['h2'] & {
    as?: 'h1' | 'h2';
    spaced?: boolean;
};

export function SectionLabel({ as = 'h2', spaced, class: extraClass, ...props }: SectionLabelProps) {
    const className = cx('section-label', spaced && 'is-spaced', extraClass);
    return as === 'h1' ? <h1 {...props} class={className} /> : <h2 {...props} class={className} />;
}

export function IconText({ class: extraClass, ...props }: JSX.IntrinsicElements['span']) {
    return <span {...props} class={cx('icon-text', extraClass)} />;
}
