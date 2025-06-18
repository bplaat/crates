/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use windows::Win32::Foundation::HWND;
use windows::core::PWSTR;

pub(crate) fn convert_pwstr_to_string(pwstr: PWSTR) -> String {
    let mut len = 0;
    while unsafe { *pwstr.0.add(len) } != 0 {
        len += 1;
    }
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(pwstr.0, len) })
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn GetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_INDEX,
) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongA(hwnd, index) }) as isize
}
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn GetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(hwnd, index) }
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn SetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_INDEX,
    value: isize,
) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongA(hwnd, index, value as i32) })
        as isize
}
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn SetWindowLong(
    hwnd: HWND,
    index: windows::Win32::UI::WindowsAndMessaging::WINDOW_LONG_PTR_INDEX,
    value: isize,
) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(hwnd, index, value) }
}
