use core::ffi::{c_int, c_void};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _Unwind_GetIP(_context: *mut c_void) -> usize {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _Unwind_Backtrace(_callback: *const c_void, _data: *mut c_void) -> c_int {
    5
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _Unwind_FindEnclosingFunction(pointer: *mut c_void) -> *mut c_void {
    pointer
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _Unwind_GetCFA(_context: *mut c_void) -> usize {
    0
}
