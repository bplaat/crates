/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, mpsc};

use bwebview::EventLoopProxy;

use crate::catalog::Catalog;
use crate::engine;
use crate::ipc::{IpcPush, ProgressBarState, send_progress, send_push};

pub(crate) enum WorkerCommand {
    Initialize,
    Scan,
    Clean(Vec<String>),
}

pub(crate) const OPERATION_IDLE: u8 = 0;
pub(crate) const OPERATION_SCAN: u8 = 1;
pub(crate) const OPERATION_CLEAN: u8 = 2;

pub(crate) fn run(
    receiver: mpsc::Receiver<WorkerCommand>,
    proxy: Arc<EventLoopProxy>,
    cancelled: Arc<AtomicBool>,
    operation_state: Arc<AtomicU8>,
    is_administrator: bool,
) {
    let catalog = match Catalog::load() {
        Ok(catalog) => catalog,
        Err(error) => {
            send_push(
                &proxy,
                IpcPush::FatalError {
                    message: error.to_string(),
                },
            );
            return;
        }
    };

    let mut scanned_cleanup_ids = HashSet::new();
    let mut has_completed_scan = false;
    for command in receiver {
        match command {
            WorkerCommand::Initialize => {
                send_push(
                    &proxy,
                    IpcPush::Catalog {
                        catalog: &catalog,
                        disk_free: None,
                        is_administrator,
                    },
                );
                send_push(
                    &proxy,
                    IpcPush::DiskFreeUpdated {
                        disk_free: engine::disk_free_space(),
                    },
                );
            }
            WorkerCommand::Scan => {
                scanned_cleanup_ids.clear();
                let rules: Vec<_> = catalog.rules().collect();
                send_progress(&proxy, ProgressBarState::Indeterminate);
                send_push(
                    &proxy,
                    IpcPush::OperationStarted {
                        operation: "scan",
                        total: rules.len(),
                    },
                );
                for (index, cleanup) in rules.iter().enumerate() {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    send_push(
                        &proxy,
                        IpcPush::CleanupProgress {
                            operation: "scan",
                            cleanup_id: &cleanup.id,
                            index,
                            total: rules.len(),
                        },
                    );
                    let result = engine::scan(cleanup, &cancelled, is_administrator);
                    if result.available
                        && (result.bytes > 0 || result.files > 0 || result.unknown_size)
                    {
                        scanned_cleanup_ids.insert(cleanup.id.clone());
                    }
                    send_push(&proxy, IpcPush::ScanResult { result });
                    send_progress(
                        &proxy,
                        ProgressBarState::Normal((index + 1) as f32 / rules.len() as f32),
                    );
                }
                let was_cancelled = cancelled.load(Ordering::Relaxed);
                has_completed_scan = !was_cancelled;
                if was_cancelled {
                    scanned_cleanup_ids.clear();
                }
                operation_state.store(OPERATION_IDLE, Ordering::Release);
                send_push(
                    &proxy,
                    IpcPush::ScanFinished {
                        cancelled: was_cancelled,
                    },
                );
                send_progress(&proxy, ProgressBarState::None);
            }
            WorkerCommand::Clean(ids) => {
                if !has_completed_scan {
                    send_push(
                        &proxy,
                        IpcPush::FatalError {
                            message: "Scan must complete before cleanup can start".to_string(),
                        },
                    );
                    operation_state.store(OPERATION_IDLE, Ordering::Release);
                    continue;
                }
                let ids: HashSet<_> = ids.into_iter().collect();
                let rules: Vec<_> = catalog
                    .rules()
                    .filter(|cleanup| {
                        ids.contains(&cleanup.id)
                            && scanned_cleanup_ids.contains(&cleanup.id)
                            && (is_administrator || !cleanup.requires_administrator)
                    })
                    .collect();
                send_progress(&proxy, ProgressBarState::Indeterminate);
                send_push(
                    &proxy,
                    IpcPush::OperationStarted {
                        operation: "clean",
                        total: rules.len(),
                    },
                );
                let mut had_errors = false;
                let mut recovered_bytes = 0;
                let mut completed_ids = HashSet::new();
                for (index, cleanup) in rules.iter().enumerate() {
                    send_push(
                        &proxy,
                        IpcPush::CleanupProgress {
                            operation: "clean",
                            cleanup_id: &cleanup.id,
                            index,
                            total: rules.len(),
                        },
                    );
                    let result = engine::clean(cleanup, is_administrator);
                    had_errors |= !result.errors.is_empty();
                    if result.errors.is_empty() {
                        completed_ids.insert(cleanup.id.clone());
                    }
                    recovered_bytes += result.cleaned_bytes;
                    send_push(&proxy, IpcPush::CleanResult { result });
                    send_progress(
                        &proxy,
                        ProgressBarState::Normal((index + 1) as f32 / rules.len() as f32),
                    );
                }
                let disk_free = engine::disk_free_space();
                send_push(&proxy, IpcPush::DiskFreeUpdated { disk_free });
                scanned_cleanup_ids.retain(|id| !completed_ids.contains(id));
                operation_state.store(OPERATION_IDLE, Ordering::Release);
                send_push(&proxy, IpcPush::CleanFinished { recovered_bytes });
                send_progress(
                    &proxy,
                    if had_errors {
                        ProgressBarState::Error(1.0)
                    } else {
                        ProgressBarState::None
                    },
                );
            }
        }
    }
}
