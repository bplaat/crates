use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[unsafe(no_mangle)]
pub extern "C" fn dlopen(_path: *const c_char, _flags: c_int) -> *mut c_void {
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn dlsym(_handle: *mut c_void, _symbol: *const c_char) -> *mut c_void {
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn dlclose(_handle: *mut c_void) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn dlerror() -> *mut c_char {
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dl_iterate_phdr(_callback: *const c_void, _data: *mut c_void) -> c_int {
    0
}
