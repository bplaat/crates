/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case)]

use windows::Win32::Foundation::HWND;

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn GetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_INDEX,
) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongA(hwnd, index) }) as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn GetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(hwnd, index) }
}

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn SetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_INDEX,
    value: isize,
) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongA(hwnd, index, value as i32) })
        as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn SetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
    value: isize,
) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(hwnd, index, value) }
}
