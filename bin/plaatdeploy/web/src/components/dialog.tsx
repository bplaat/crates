/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import { Button, DangerButton, IconButton, SecondaryButton } from './button.tsx';
import { FormField } from './form.tsx';
import { FormInput } from './input.tsx';
import { CloseIcon, DeleteIcon } from './icons.tsx';

interface DialogProps {
    title: string;
    onClose: () => void;
    children: ComponentChildren;
}

export function Dialog({ title, onClose, children }: DialogProps) {
    const dialogRef = useRef<HTMLDialogElement>(null);

    useEffect(() => {
        const dialog = dialogRef.current;
        if (dialog && !dialog.open) dialog.showModal();
        return () => {
            if (dialog?.open) dialog.close();
        };
    }, []);

    return (
        <dialog
            ref={dialogRef}
            class="modal"
            onCancel={(event) => {
                event.preventDefault();
                onClose();
            }}
            onMouseDown={(event) => {
                if (event.target === dialogRef.current) onClose();
            }}
        >
            <div class="modal-card">
                <div class="modal-card-head">
                    <h2>{title}</h2>
                    <IconButton type="button" onClick={onClose} aria-label="Close">
                        <CloseIcon class="is-md" />
                    </IconButton>
                </div>
                <div class="modal-card-body">{children}</div>
            </div>
        </dialog>
    );
}

interface ConfirmDialogProps {
    title: string;
    message: string;
    confirmLabel: string;
    // When set, the user must type this exact value before the confirm button is enabled.
    confirmationText?: string;
    // Confirm button styling; defaults to a destructive (danger) action.
    danger?: boolean;
    // Icon shown on the confirm button; defaults to a delete icon for danger actions.
    icon?: ComponentChildren;
    onConfirm: () => void;
    onClose: () => void;
}

export function ConfirmDialog({
    title,
    message,
    confirmLabel,
    confirmationText,
    danger = true,
    icon,
    onConfirm,
    onClose,
}: ConfirmDialogProps) {
    const [typedText, setTypedText] = useState('');
    const gated = confirmationText !== undefined && confirmationText !== '';
    const disabled = gated && typedText.trim() !== confirmationText;
    const ConfirmButton = danger ? DangerButton : Button;
    const confirmIcon = icon ?? (danger ? <DeleteIcon class="is-sm" /> : undefined);

    return (
        <Dialog title={title} onClose={onClose}>
            <p class="modal-text">{message}</p>
            {gated && (
                <FormField id="confirm-dialog-input" label={`Type "${confirmationText}" to confirm`}>
                    <FormInput
                        id="confirm-dialog-input"
                        type="text"
                        value={typedText}
                        placeholder={confirmationText}
                        autocomplete="off"
                        autofocus
                        onInput={(e) => setTypedText((e.target as HTMLInputElement).value)}
                    />
                </FormField>
            )}
            <div class="buttons">
                <SecondaryButton type="button" onClick={onClose}>
                    Cancel
                </SecondaryButton>
                <ConfirmButton type="button" onClick={onConfirm} disabled={disabled}>
                    {confirmIcon}
                    {confirmLabel}
                </ConfirmButton>
            </div>
        </Dialog>
    );
}
