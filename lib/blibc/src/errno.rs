use core::ffi::c_int;

pub(crate) fn convert_syscall_result(result: isize) -> isize {
    if (-4095..0).contains(&result) {
        set_errno(-result as c_int);
        -1
    } else {
        result
    }
}

pub(crate) fn set_errno(error: c_int) {
    unsafe { __errno_location().write(error) };
}

#[unsafe(no_mangle)]
pub extern "C" fn __errno_location() -> *mut c_int {
    crate::thread::errno_location()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __xpg_strerror_r(_error: c_int, buffer: *mut i8, length: usize) -> c_int {
    if length != 0 {
        unsafe { buffer.write(0) };
    }
    0
}

#[cfg(test)]
mod tests {
    use super::{__errno_location, set_errno};

    #[test]
    fn errno_is_thread_local() {
        set_errno(7);
        std::thread::spawn(|| {
            set_errno(9);
            assert_eq!(unsafe { __errno_location().read() }, 9);
        })
        .join()
        .expect("errno test thread panicked");
        assert_eq!(unsafe { __errno_location().read() }, 7);
    }
}
