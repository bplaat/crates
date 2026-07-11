/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { Button, DangerButton, IconButton, SecondaryButton } from './button.tsx';
import { FormActions, FormField } from './form.tsx';
import { FormInput } from './input.tsx';
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
    // When set, the user must type this exact value before the confirm button is enabled.
    confirmText?: string;
    // Confirm button styling; defaults to a destructive (danger) action.
    danger?: boolean;
    // Icon shown on the confirm button; defaults to a delete icon.
    icon?: ComponentChildren;
    onConfirm: () => void;
    onClose: () => void;
}

export function ConfirmDialog({
    title,
    message,
    confirmLabel,
    confirmText,
    danger = true,
    icon,
    onConfirm,
    onClose,
}: ConfirmDialogProps) {
    const [typed, setTyped] = useState('');
    const gated = confirmText !== undefined && confirmText !== '';
    const disabled = gated && typed.trim() !== confirmText;
    const ConfirmButton = danger ? DangerButton : Button;

    return (
        <Dialog title={title} onClose={onClose}>
            <div class="form">
                <p class="modal-text">{message}</p>
                {gated && (
                    <FormField id="confirm-text" label={t('dialog.type_to_confirm', confirmText!)}>
                        <FormInput
                            id="confirm-text"
                            type="text"
                            value={typed}
                            placeholder={confirmText}
                            autoComplete="off"
                            onInput={(e) => setTyped((e.target as HTMLInputElement).value)}
                        />
                    </FormField>
                )}
                <FormActions class="is-flush">
                    <SecondaryButton onClick={onClose}>{t('dialog.cancel')}</SecondaryButton>
                    <ConfirmButton onClick={onConfirm} disabled={disabled}>
                        {icon ?? <DeleteOutlineIcon class="is-sm" />}
                        {confirmLabel}
                    </ConfirmButton>
                </FormActions>
            </div>
        </Dialog>
    );
}
