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
        <div class="modal" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
            <div role="dialog" aria-modal="true" class="modal-card">
                <div class="modal-card-head">
                    <h2 class="modal-card-title">{title}</h2>
                    <IconButton onClick={onClose} class="has-text-muted">
                        <CloseIcon class="is-md" />
                    </IconButton>
                </div>
                <div class="modal-card-body">{children}</div>
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
            <p class="modal-text">{message}</p>
            <FormActions class="is-flush">
                <SecondaryButton onClick={onClose}>{t('dialog.cancel')}</SecondaryButton>
                <DangerButton onClick={onConfirm}>
                    <DeleteOutlineIcon class="is-sm" />
                    {confirmLabel}
                </DangerButton>
            </FormActions>
        </Dialog>
    );
}
