use core::ffi::{c_int, c_void};
use core::ptr;

use crate::errno::set_errno;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn signal(_signal: c_int, _handler: usize) -> usize {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaction(
    _signal: c_int,
    _action: *const c_void,
    _old_action: *mut c_void,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaltstack(_stack: *const c_void, _old_stack: *mut c_void) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigemptyset(set: *mut c_void) -> c_int {
    unsafe { ptr::write_bytes(set.cast::<u8>(), 0, 128) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaddset(set: *mut c_void, signal: c_int) -> c_int {
    if !(1..=1024).contains(&signal) {
        set_errno(22);
        return -1;
    }
    let index = (signal - 1) as usize;
    unsafe {
        let byte = set.cast::<u8>().add(index / 8);
        byte.write(byte.read() | (1 << (index % 8)));
    }
    0
}
