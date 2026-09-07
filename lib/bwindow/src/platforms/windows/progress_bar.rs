/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::null_mut;

use super::headers::*;
use crate::WindowsProgressBarState;

/// The progress a window asked for, applied once its taskbar button exists
#[derive(Default)]
pub(super) struct ProgressBar {
    progress: Option<f32>,
    state: WindowsProgressBarState,
    button_ready: bool,
}

impl ProgressBar {
    /// Stores the requested progress and shows it when the taskbar button is ready
    pub(super) fn set(
        &mut self,
        hwnd: HWND,
        progress: Option<f32>,
        state: WindowsProgressBarState,
    ) {
        self.progress = progress;
        self.state = state;
        if self.button_ready {
            set_progress_bar(hwnd, progress, state);
        }
    }

    /// Shows the stored progress now that Explorer has created the taskbar button
    pub(super) fn button_created(&mut self, hwnd: HWND) {
        self.button_ready = true;
        set_progress_bar(hwnd, self.progress, self.state);
    }
}

/// Shows, updates or hides the taskbar button progress of a window
fn set_progress_bar(hwnd: HWND, progress: Option<f32>, state: WindowsProgressBarState) {
    let mut taskbar = null_mut::<TaskbarList3>();
    let result = unsafe {
        CoCreateInstance(
            &CLSID_TASKBAR_LIST,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_TASKBAR_LIST3,
            &mut taskbar as *mut _ as *mut *mut c_void,
        )
    };
    if result < 0 || taskbar.is_null() {
        return;
    }
    unsafe {
        let vtable = &*(*taskbar).vtable;
        if (vtable.hr_init)(taskbar) >= 0 {
            let taskbar_state = match (progress, state) {
                (None, _) => 0,
                (Some(_), WindowsProgressBarState::Normal) => 2,
                (Some(_), WindowsProgressBarState::Error) => 4,
                (Some(_), WindowsProgressBarState::Paused) => 8,
                (Some(_), WindowsProgressBarState::Indeterminate) => 1,
            };
            if let (Some(progress), false) = (
                progress,
                matches!(state, WindowsProgressBarState::Indeterminate),
            ) {
                let progress = progress.clamp(0.0, 1.0);
                _ = (vtable.set_progress_value)(taskbar, hwnd, (progress * 1000.0) as u64, 1000);
            }
            _ = (vtable.set_progress_state)(taskbar, hwnd, taskbar_state);
        }
        _ = (vtable.release)(taskbar);
    }
}
