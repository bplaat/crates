/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import type { ComponentChildren } from 'preact';

interface DialogProps {
    title: string;
    onClose: () => void;
    children: ComponentChildren;
}

export function Dialog({ title, onClose, children }: DialogProps) {
    return (
        <dialog open class="app-dialog" onCancel={onClose}>
            <div class="app-dialog-card">
                <div class="app-dialog-header">
                    <h2>{title}</h2>
                    <button class="btn btn-secondary btn-sm" type="button" onClick={onClose}>
                        Close
                    </button>
                </div>
                <div class="app-dialog-body">{children}</div>
            </div>
        </dialog>
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
            <p class="dialog-message">{message}</p>
            <div class="dialog-actions">
                <button class="btn btn-secondary" type="button" onClick={onClose}>
                    Cancel
                </button>
                <button class="btn btn-danger" type="button" onClick={onConfirm}>
                    {confirmLabel}
                </button>
            </div>
        </Dialog>
    );
}
