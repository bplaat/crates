use core::ffi::{c_char, c_int};
use core::ptr;

use crate::alloc::{free, malloc};
use crate::errno::convert_syscall_result;
use crate::io::{close, lseek64};
use crate::syscall::{syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_GETDENTS64: usize = 217;
        const SYS_OPENAT: usize = 257;
        const O_DIRECTORY: usize = 0x10000;
    }
    target_arch = "aarch64" => {
        const SYS_OPENAT: usize = 56;
        const SYS_GETDENTS64: usize = 61;
        const O_DIRECTORY: usize = 0x4000;
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

const AT_FDCWD: isize = -100;
const O_RDONLY: usize = 0;
const O_CLOEXEC: usize = 0x80000;
const BUFFER_SIZE: usize = 4096;

#[repr(C)]
pub struct DIR {
    fd: c_int,
    position: usize,
    length: usize,
    buffer: [u8; BUFFER_SIZE],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct dirent64 {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [c_char; 0],
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn opendir(path: *const c_char) -> *mut DIR {
    let fd = convert_syscall_result(unsafe {
        syscall6(
            SYS_OPENAT,
            AT_FDCWD as usize,
            path as usize,
            O_RDONLY | O_CLOEXEC | O_DIRECTORY,
            0,
            0,
            0,
        )
    }) as c_int;
    if fd < 0 {
        return ptr::null_mut();
    }
    let directory = unsafe { fdopendir(fd) };
    if directory.is_null() {
        unsafe { close(fd) };
    }
    directory
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fdopendir(fd: c_int) -> *mut DIR {
    let directory = unsafe { malloc(core::mem::size_of::<DIR>()) }.cast::<DIR>();
    if directory.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        directory.write(DIR {
            fd,
            position: 0,
            length: 0,
            buffer: [0; BUFFER_SIZE],
        });
    }
    directory
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn readdir64(directory: *mut DIR) -> *mut dirent64 {
    if directory.is_null() {
        return ptr::null_mut();
    }
    let directory = unsafe { &mut *directory };
    if directory.position >= directory.length {
        let length = convert_syscall_result(unsafe {
            syscall3(
                SYS_GETDENTS64,
                directory.fd as usize,
                directory.buffer.as_mut_ptr() as usize,
                BUFFER_SIZE,
            )
        });
        if length <= 0 {
            return ptr::null_mut();
        }
        directory.position = 0;
        directory.length = length as usize;
    }
    let entry = unsafe { directory.buffer.as_mut_ptr().add(directory.position) }.cast::<dirent64>();
    let record_length = unsafe { (*entry).d_reclen as usize };
    if record_length == 0 || directory.position + record_length > directory.length {
        return ptr::null_mut();
    }
    directory.position += record_length;
    entry
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dirfd(directory: *mut DIR) -> c_int {
    unsafe { (*directory).fd }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn closedir(directory: *mut DIR) -> c_int {
    if directory.is_null() {
        return -1;
    }
    let result = unsafe { close((*directory).fd) };
    unsafe { free(directory.cast()) };
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rewinddir(directory: *mut DIR) {
    if !directory.is_null() && unsafe { lseek64((*directory).fd, 0, 0) } == 0 {
        unsafe {
            (*directory).position = 0;
            (*directory).length = 0;
        }
    }
}
