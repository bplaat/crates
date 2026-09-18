use core::ffi::{c_int, c_void};

use crate::errno::convert_syscall_result;
use crate::syscall::{syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_MMAP: usize = 9;
        const SYS_MPROTECT: usize = 10;
        const SYS_MUNMAP: usize = 11;
        const SYS_MREMAP: usize = 25;
    }
    target_arch = "aarch64" => {
        const SYS_MMAP: usize = 222;
        const SYS_MPROTECT: usize = 226;
        const SYS_MUNMAP: usize = 215;
        const SYS_MREMAP: usize = 216;
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mmap64(
    address: *mut c_void,
    length: usize,
    protection: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_MMAP,
            address as usize,
            length,
            protection as usize,
            flags as usize,
            fd as usize,
            offset as usize,
        )
    }) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mmap(
    address: *mut c_void,
    length: usize,
    protection: c_int,
    flags: c_int,
    fd: c_int,
    offset: i64,
) -> *mut c_void {
    unsafe { mmap64(address, length, protection, flags, fd, offset) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn munmap(address: *mut c_void, length: usize) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_MUNMAP, address as usize, length, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mprotect(address: *mut c_void, length: usize, protection: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_MPROTECT, address as usize, length, protection as usize)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mremap(
    old_address: *mut c_void,
    old_size: usize,
    new_size: usize,
    flags: c_int,
    new_address: *mut c_void,
) -> *mut c_void {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_MREMAP,
            old_address as usize,
            old_size,
            new_size,
            flags as usize,
            new_address as usize,
            0,
        )
    }) as *mut c_void
}

#[cfg(test)]
mod tests {
    use super::mmap;
    use crate::errno::__errno_location;

    #[test]
    fn mmap_failure_uses_libc_error_semantics() {
        let result = unsafe { mmap(core::ptr::null_mut(), 0, 0, 0x22, -1, 0) };
        assert_eq!(result as usize, usize::MAX);
        assert_eq!(unsafe { __errno_location().read() }, 22);
    }
}
