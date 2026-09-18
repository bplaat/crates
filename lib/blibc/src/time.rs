use core::ffi::{c_char, c_int, c_long};

use crate::errno::convert_syscall_result;
use crate::syscall::{syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_GETTIMEOFDAY: usize = 96;
        const SYS_NANOSLEEP: usize = 35;
        const SYS_CLOCK_GETTIME: usize = 228;
        const SYS_CLOCK_NANOSLEEP: usize = 230;
    }
    target_arch = "aarch64" => {
        const SYS_GETTIMEOFDAY: usize = 169;
        const SYS_NANOSLEEP: usize = 101;
        const SYS_CLOCK_GETTIME: usize = 113;
        const SYS_CLOCK_NANOSLEEP: usize = 115;
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct timespec {
    pub tv_sec: i64,
    pub tv_nsec: c_long,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct timeval {
    pub tv_sec: i64,
    pub tv_usec: c_long,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clock: c_int, time: *mut timespec) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_CLOCK_GETTIME, clock as usize, time as usize, 0) })
        as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gettimeofday(
    time: *mut timeval,
    _timezone: *mut core::ffi::c_void,
) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_GETTIMEOFDAY, time as usize, 0, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nanosleep(requested: *const timespec, remaining: *mut timespec) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_NANOSLEEP, requested as usize, remaining as usize, 0)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn time(output: *mut i64) -> i64 {
    let mut value = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if unsafe { gettimeofday(&raw mut value, core::ptr::null_mut()) } != 0 {
        return -1;
    }
    if !output.is_null() {
        unsafe { output.write(value.tv_sec) };
    }
    value.tv_sec
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn localtime(_value: *const i64) -> *mut tm {
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_nanosleep(
    clock: c_int,
    flags: c_int,
    requested: *const timespec,
    remaining: *mut timespec,
) -> c_int {
    let result = unsafe {
        syscall6(
            SYS_CLOCK_NANOSLEEP,
            clock as usize,
            flags as usize,
            requested as usize,
            remaining as usize,
            0,
            0,
        )
    };
    if (-4095..0).contains(&result) {
        -result as c_int
    } else {
        result as c_int
    }
}
