/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { useEffect } from 'preact/hooks';
import { DangerButton, IconButton, SecondaryButton } from './button.tsx';
import { FormActions } from './form.tsx';
import { CloseIcon, DeleteOutlineIcon } from './icons.tsx';
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
                        <CloseIcon class="w-5 h-5" />
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
                    <DeleteOutlineIcon class="w-4 h-4" />
                    {confirmLabel}
                </DangerButton>
            </FormActions>
        </Dialog>
    );
}
