/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { useEffect } from 'preact/hooks';
import { DangerButton, IconButton, SecondaryButton } from './button.tsx';
import { FormActions } from './form.tsx';
import { t } from '../services/i18n.service.ts';

interface DialogProps {
    title: string;
    onClose: () => void;
    children: ComponentChildren;
}

export function Dialog({ title, onClose, children }: DialogProps) {
    useEffect(() => {
        function onKey(e: KeyboardEvent) {
            if (e.key === 'Escape') onClose();
        }
        document.addEventListener('keydown', onKey);
        return () => document.removeEventListener('keydown', onKey);
    }, [onClose]);

    return (
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60"
            onMouseDown={(e) => e.target === e.currentTarget && onClose()}
        >
            <div
                role="dialog"
                aria-modal="true"
                class="bg-white dark:bg-zinc-800 rounded-2xl shadow-xl w-full max-w-md mx-4"
            >
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-zinc-700">
                    <h2 class="text-base font-semibold text-gray-800 dark:text-gray-100">{title}</h2>
                    <IconButton onClick={onClose} class="text-gray-500 dark:text-gray-400">
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                        </svg>
                    </IconButton>
                </div>
                <div class="px-6 py-5">{children}</div>
            </div>
        </div>
    );
}

interface ConfirmDialogProps {
    title: string;
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
    onClose: () => void;
}

export function ConfirmDialog({ title, message, confirmLabel, onConfirm, onClose }: ConfirmDialogProps) {
    return (
        <Dialog title={title} onClose={onClose}>
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-5">{message}</p>
            <FormActions class="pt-0">
                <SecondaryButton onClick={onClose}>{t('dialog.cancel')}</SecondaryButton>
                <DangerButton onClick={onConfirm}>
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z" />
                    </svg>
                    {confirmLabel}
                </DangerButton>
            </FormActions>
        </Dialog>
    );
}
