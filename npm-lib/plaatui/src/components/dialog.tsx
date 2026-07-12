/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { type ComponentChildren } from 'preact';
import { useLayoutEffect, useRef, useState } from 'preact/hooks';
import './dialog.css';
import { Button, DangerButton, IconButton, SecondaryButton } from './button.tsx';
import { FormActions, FormField } from './form.tsx';
import { FormInput } from './input.tsx';
import { Icon } from './icons.tsx';

export interface DialogProps {
    title: string;
    closeLabel?: string;
    onClose: () => void;
    children: ComponentChildren;
}

export function Dialog({ title, closeLabel = 'Close', onClose, children }: DialogProps) {
    const dialogRef = useRef<HTMLDialogElement>(null);

    // Lock background scroll while open. Reserve the scrollbar's width as padding so
    // hiding it shifts neither the page content nor the fixed, centered overlay.
    useLayoutEffect(() => {
        const dialog = dialogRef.current;
        dialog?.showModal();

        const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
        const { overflow, paddingRight } = document.body.style;
        document.body.style.overflow = 'hidden';
        if (scrollbarWidth > 0) {
            document.body.style.paddingRight = `${scrollbarWidth}px`;
            dialog?.style.setProperty('padding-right', `${scrollbarWidth}px`);
        }
        return () => {
            dialog?.close();
            document.body.style.overflow = overflow;
            document.body.style.paddingRight = paddingRight;
        };
    }, []);

    return (
        <dialog
            ref={dialogRef}
            class="modal"
            onCancel={(e) => {
                e.preventDefault();
                onClose();
            }}
            onMouseDown={(e) => e.target === e.currentTarget && onClose()}
        >
            <div class="modal-card">
                <div class="modal-card-head">
                    <h2 class="modal-card-title">{title}</h2>
                    <IconButton type="button" onClick={onClose} class="has-text-muted" title={closeLabel}>
                        <Icon type="close" class="is-md" />
                    </IconButton>
                </div>
                <div class="modal-card-body">{children}</div>
            </div>
        </dialog>
    );
}

export interface ConfirmDialogProps {
    title: string;
    message: string;
    confirmLabel: string;
    cancelLabel: string;
    typeToConfirmLabel?: (confirmText: string) => string;
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
    cancelLabel,
    typeToConfirmLabel,
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
                    <FormField
                        id="confirm-text"
                        label={typeToConfirmLabel ? typeToConfirmLabel(confirmText!) : confirmText!}
                    >
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
                    <SecondaryButton onClick={onClose}>{cancelLabel}</SecondaryButton>
                    <ConfirmButton onClick={onConfirm} disabled={disabled}>
                        {icon ?? <Icon type="delete-outline" class="is-sm" />}
                        {confirmLabel}
                    </ConfirmButton>
                </FormActions>
            </div>
        </Dialog>
    );
}
