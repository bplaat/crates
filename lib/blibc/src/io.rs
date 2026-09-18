use core::ffi::{c_char, c_int, c_long, c_void};

use crate::errno::convert_syscall_result;
use crate::syscall::{syscall1, syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_READ: usize = 0;
        const SYS_WRITE: usize = 1;
        const SYS_CLOSE: usize = 3;
        const SYS_FSTAT: usize = 5;
        const SYS_POLL: usize = 7;
        const SYS_LSEEK: usize = 8;
        const SYS_IOCTL: usize = 16;
        const SYS_PREAD64: usize = 17;
        const SYS_PWRITE64: usize = 18;
        const SYS_READV: usize = 19;
        const SYS_WRITEV: usize = 20;
        const SYS_DUP: usize = 32;
        const SYS_DUP2: usize = 33;
        const SYS_SENDFILE: usize = 40;
        const SYS_FCNTL: usize = 72;
        const SYS_FSYNC: usize = 74;
        const SYS_FTRUNCATE: usize = 77;
        const SYS_FCHMOD: usize = 91;
        const SYS_FCHOWN: usize = 93;
        const SYS_OPENAT: usize = 257;
        const SYS_MKDIRAT: usize = 258;
        const SYS_NEWFSTATAT: usize = 262;
        const SYS_UNLINKAT: usize = 263;
        const SYS_SYMLINKAT: usize = 266;
        const SYS_READLINKAT: usize = 267;
        const SYS_FACCESSAT: usize = 269;
        const SYS_SPLICE: usize = 275;
        const SYS_UTIMENSAT: usize = 280;

        unsafe fn poll_impl(fds: *mut pollfd, count: usize, timeout: c_int) -> c_int {
            unsafe { syscall3(SYS_POLL, fds as usize, count, timeout as usize) as c_int }
        }
    }
    target_arch = "aarch64" => {
        const SYS_DUP: usize = 23;
        const SYS_DUP3: usize = 24;
        const SYS_FCNTL: usize = 25;
        const SYS_IOCTL: usize = 29;
        const SYS_MKDIRAT: usize = 34;
        const SYS_UNLINKAT: usize = 35;
        const SYS_SYMLINKAT: usize = 36;
        const SYS_FTRUNCATE: usize = 46;
        const SYS_FACCESSAT: usize = 48;
        const SYS_FCHMOD: usize = 52;
        const SYS_FCHOWN: usize = 55;
        const SYS_OPENAT: usize = 56;
        const SYS_CLOSE: usize = 57;
        const SYS_LSEEK: usize = 62;
        const SYS_READ: usize = 63;
        const SYS_WRITE: usize = 64;
        const SYS_READV: usize = 65;
        const SYS_WRITEV: usize = 66;
        const SYS_PREAD64: usize = 67;
        const SYS_PWRITE64: usize = 68;
        const SYS_SENDFILE: usize = 71;
        const SYS_PPOLL: usize = 73;
        const SYS_SPLICE: usize = 76;
        const SYS_READLINKAT: usize = 78;
        const SYS_NEWFSTATAT: usize = 79;
        const SYS_FSTAT: usize = 80;
        const SYS_FSYNC: usize = 82;
        const SYS_UTIMENSAT: usize = 88;

        unsafe fn poll_impl(fds: *mut pollfd, count: usize, timeout: c_int) -> c_int {
            let timeout_value = crate::time::timespec {
                tv_sec: (timeout / 1000) as i64,
                tv_nsec: ((timeout % 1000) * 1_000_000) as i64,
            };
            let timeout_pointer = if timeout < 0 {
                core::ptr::null()
            } else {
                &raw const timeout_value
            };
            unsafe {
                syscall6(
                    SYS_PPOLL,
                    fds as usize,
                    count,
                    timeout_pointer as usize,
                    0,
                    0,
                    0,
                ) as c_int
            }
        }
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

const AT_FDCWD: isize = -100;
const AT_REMOVEDIR: usize = 0x200;
const O_CREAT: c_int = 0x40;
const O_TMPFILE: c_int = 0x410000;
const TCGETS: usize = 0x5401;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct iovec {
    pub iov_base: *mut c_void,
    pub iov_len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct pollfd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write(fd: c_int, buffer: *const c_void, count: usize) -> isize {
    convert_syscall_result(unsafe { syscall3(SYS_WRITE, fd as usize, buffer as usize, count) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn writev(fd: c_int, vectors: *const iovec, count: c_int) -> isize {
    convert_syscall_result(unsafe {
        syscall3(SYS_WRITEV, fd as usize, vectors as usize, count as usize)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn readv(fd: c_int, vectors: *mut iovec, count: c_int) -> isize {
    convert_syscall_result(unsafe {
        syscall3(SYS_READV, fd as usize, vectors as usize, count as usize)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn poll(fds: *mut pollfd, count: usize, timeout: c_int) -> c_int {
    convert_syscall_result(unsafe { poll_impl(fds, count, timeout) } as isize) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fcntl(fd: c_int, command: c_int, argument: c_long) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_FCNTL, fd as usize, command as usize, argument as usize)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fcntl64(fd: c_int, command: c_int, argument: c_long) -> c_int {
    unsafe { fcntl(fd, command, argument) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dup(fd: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall1(SYS_DUP, fd as usize) }) as c_int
}

cfg_select! {
    target_arch = "x86_64" => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dup2(fd: c_int, target: c_int) -> c_int {
            convert_syscall_result(unsafe { syscall3(SYS_DUP2, fd as usize, target as usize, 0) })
                as c_int
        }
    }
    target_arch = "aarch64" => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dup2(fd: c_int, target: c_int) -> c_int {
            if fd == target {
                const F_GETFD: c_int = 1;
                return if unsafe { fcntl(fd, F_GETFD, 0) } < 0 {
                    -1
                } else {
                    target
                };
            }
            convert_syscall_result(unsafe { syscall3(SYS_DUP3, fd as usize, target as usize, 0) })
                as c_int
        }
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fd: c_int, buffer: *mut c_void, count: usize) -> isize {
    convert_syscall_result(unsafe { syscall3(SYS_READ, fd as usize, buffer as usize, count) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall1(SYS_CLOSE, fd as usize) }) as c_int
}

unsafe fn open_impl(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_OPENAT,
            AT_FDCWD as usize,
            path as usize,
            flags as usize,
            mode as usize,
            0,
            0,
        )
    }) as c_int
}

unsafe fn open_mode(flags: c_int, arguments: &mut core::ffi::VaList<'_>) -> c_int {
    if flags & O_CREAT != 0 || flags & O_TMPFILE == O_TMPFILE {
        unsafe { arguments.next_arg() }
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, mut arguments: ...) -> c_int {
    let mode = unsafe { open_mode(flags, &mut arguments) };
    unsafe { open_impl(path, flags, mode) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn open64(path: *const c_char, flags: c_int, mut arguments: ...) -> c_int {
    let mode = unsafe { open_mode(flags, &mut arguments) };
    unsafe { open_impl(path, flags, mode) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn openat64(
    directory: c_int,
    path: *const c_char,
    flags: c_int,
    mut arguments: ...
) -> c_int {
    let mode = unsafe { open_mode(flags, &mut arguments) };
    convert_syscall_result(unsafe {
        syscall6(
            SYS_OPENAT,
            directory as usize,
            path as usize,
            flags as usize,
            mode as usize,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat64(path: *const c_char, output: *mut c_void) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_NEWFSTATAT,
            AT_FDCWD as usize,
            path as usize,
            output as usize,
            0,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstat64(fd: c_int, output: *mut c_void) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_FSTAT, fd as usize, output as usize, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstatat64(
    directory: c_int,
    path: *const c_char,
    output: *mut c_void,
    flags: c_int,
) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_NEWFSTATAT,
            directory as usize,
            path as usize,
            output as usize,
            flags as usize,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
    convert_syscall_result(unsafe {
        syscall3(SYS_LSEEK, fd as usize, offset as usize, whence as usize)
    }) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall1(SYS_FSYNC, fd as usize) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pread64(
    fd: c_int,
    buffer: *mut c_void,
    count: usize,
    offset: i64,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_PREAD64,
            fd as usize,
            buffer as usize,
            count,
            offset as usize,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pwrite64(
    fd: c_int,
    buffer: *const c_void,
    count: usize,
    offset: i64,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_PWRITE64,
            fd as usize,
            buffer as usize,
            count,
            offset as usize,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftruncate64(fd: c_int, length: i64) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_FTRUNCATE, fd as usize, length as usize, 0) })
        as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fchmod(fd: c_int, mode: u32) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_FCHMOD, fd as usize, mode as usize, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fchown(fd: c_int, owner: u32, group: u32) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_FCHOWN, fd as usize, owner as usize, group as usize)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn access(path: *const c_char, mode: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_FACCESSAT,
            AT_FDCWD as usize,
            path as usize,
            mode as usize,
            0,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: u32) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_MKDIRAT, AT_FDCWD as usize, path as usize, mode as usize)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(SYS_UNLINKAT, AT_FDCWD as usize, path as usize, AT_REMOVEDIR)
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ioctl(fd: c_int, request: usize, argument: *mut c_void) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_IOCTL, fd as usize, request, argument as usize) })
        as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    let mut termios = [0u8; 64];
    (unsafe { ioctl(fd, TCGETS, termios.as_mut_ptr().cast()) } == 0) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_UNLINKAT, AT_FDCWD as usize, path as usize, 0) })
        as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlinkat(directory: c_int, path: *const c_char, flags: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(
            SYS_UNLINKAT,
            directory as usize,
            path as usize,
            flags as usize,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn symlink(target: *const c_char, link: *const c_char) -> c_int {
    convert_syscall_result(unsafe {
        syscall3(
            SYS_SYMLINKAT,
            target as usize,
            AT_FDCWD as usize,
            link as usize,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sendfile64(
    output: c_int,
    input: c_int,
    offset: *mut i64,
    count: usize,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_SENDFILE,
            output as usize,
            input as usize,
            offset as usize,
            count,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn splice(
    input: c_int,
    input_offset: *mut i64,
    output: c_int,
    output_offset: *mut i64,
    length: usize,
    flags: u32,
) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_SPLICE,
            input as usize,
            input_offset as usize,
            output as usize,
            output_offset as usize,
            length,
            flags as usize,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lstat64(path: *const c_char, output: *mut c_void) -> c_int {
    const AT_SYMLINK_NOFOLLOW: usize = 0x100;
    convert_syscall_result(unsafe {
        syscall6(
            SYS_NEWFSTATAT,
            AT_FDCWD as usize,
            path as usize,
            output as usize,
            AT_SYMLINK_NOFOLLOW,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn readlink(path: *const c_char, buffer: *mut c_char, size: usize) -> isize {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_READLINKAT,
            AT_FDCWD as usize,
            path as usize,
            buffer as usize,
            size,
            0,
            0,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utimes(path: *const c_char, values: *const c_void) -> c_int {
    let mut times = [crate::time::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    }; 2];
    let times_pointer = if values.is_null() {
        core::ptr::null()
    } else {
        let source = values.cast::<crate::time::timeval>();
        for (index, time) in times.iter_mut().enumerate() {
            let value = unsafe { &*source.add(index) };
            time.tv_sec = value.tv_sec;
            time.tv_nsec = value.tv_usec * 1_000;
        }
        times.as_ptr()
    };
    convert_syscall_result(unsafe {
        syscall6(
            SYS_UTIMENSAT,
            AT_FDCWD as usize,
            path as usize,
            times_pointer as usize,
            0,
            0,
            0,
        )
    }) as c_int
}
