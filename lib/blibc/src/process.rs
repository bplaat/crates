use core::ffi::{c_char, c_int, c_void};

use crate::env::getenv;
use crate::errno::convert_syscall_result;
use crate::syscall::{syscall1, syscall3, syscall6};

unsafe extern "C" {
    static environ: *const *const c_char;
}

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_GETPID: usize = 39;
        const SYS_FORK: usize = 57;
        const SYS_EXECVE: usize = 59;
        const SYS_WAIT4: usize = 61;
        const SYS_CHDIR: usize = 80;
        const SYS_GETUID: usize = 102;
        const SYS_SETUID: usize = 105;
        const SYS_SETGID: usize = 106;
        const SYS_SETPGID: usize = 109;
        const SYS_SETSID: usize = 112;
        const SYS_SETGROUPS: usize = 116;
        const SYS_CHROOT: usize = 161;
        const SYS_WAITID: usize = 247;
        const SYS_PIPE2: usize = 293;
        const SYS_GETEUID: usize = 107;
        const SYS_EXIT_GROUP: usize = 231;
        const SYS_PAUSE: usize = 34;

        unsafe fn pause_impl() -> c_int {
            convert_syscall_result(unsafe { syscall1(SYS_PAUSE, 0) }) as c_int
        }
    }
    target_arch = "aarch64" => {
        const SYS_GETEUID: usize = 175;
        const SYS_GETPID: usize = 172;
        const SYS_GETUID: usize = 174;
        const SYS_CHDIR: usize = 49;
        const SYS_CHROOT: usize = 51;
        const SYS_PIPE2: usize = 59;
        const SYS_WAITID: usize = 95;
        const SYS_SETGID: usize = 144;
        const SYS_SETUID: usize = 146;
        const SYS_SETPGID: usize = 154;
        const SYS_SETSID: usize = 157;
        const SYS_SETGROUPS: usize = 159;
        const SYS_CLONE: usize = 220;
        const SYS_EXECVE: usize = 221;
        const SYS_WAIT4: usize = 260;
        const SYS_EXIT_GROUP: usize = 94;
        const SYS_PPOLL: usize = 73;

        unsafe fn pause_impl() -> c_int {
            convert_syscall_result(unsafe { syscall6(SYS_PPOLL, 0, 0, 0, 0, 0, 0) }) as c_int
        }
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

fn exit_impl(status: c_int) -> ! {
    unsafe {
        syscall1(SYS_EXIT_GROUP, status as usize);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    exit_impl(134)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exit(status: c_int) -> ! {
    exit_impl(status)
}

#[unsafe(no_mangle)]
pub extern "C" fn _exit(status: c_int) -> ! {
    exit_impl(status)
}

#[unsafe(no_mangle)]
pub extern "C" fn getpid() -> c_int {
    unsafe { syscall1(SYS_GETPID, 0) as c_int }
}

#[unsafe(no_mangle)]
pub extern "C" fn geteuid() -> u32 {
    unsafe { syscall1(SYS_GETEUID, 0) as u32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn getuid() -> u32 {
    unsafe { syscall1(SYS_GETUID, 0) as u32 }
}

cfg_select! {
    target_arch = "x86_64" => {
        #[unsafe(no_mangle)]
        pub extern "C" fn fork() -> c_int {
            convert_syscall_result(unsafe { syscall1(SYS_FORK, 0) }) as c_int
        }
    }
    target_arch = "aarch64" => {
        #[unsafe(no_mangle)]
        pub extern "C" fn fork() -> c_int {
            const SIGCHLD: usize = 17;
            convert_syscall_result(unsafe { syscall6(SYS_CLONE, SIGCHLD, 0, 0, 0, 0, 0) }) as c_int
        }
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn execvp(file: *const c_char, arguments: *const *const c_char) -> c_int {
    let mut cursor = file;
    while unsafe { cursor.read() } != 0 {
        if unsafe { cursor.read() } == b'/' as c_char {
            return convert_syscall_result(unsafe {
                syscall3(
                    SYS_EXECVE,
                    file as usize,
                    arguments as usize,
                    environ as usize,
                )
            }) as c_int;
        }
        cursor = unsafe { cursor.add(1) };
    }

    let path = unsafe { getenv(c"PATH".as_ptr()) };
    let path = if path.is_null() {
        c"/bin:/usr/bin".as_ptr()
    } else {
        path
    };
    let mut directory = path;
    let mut candidate = [0 as c_char; 4096];
    loop {
        let mut output = 0;
        while unsafe { directory.read() } != 0
            && unsafe { directory.read() } != b':' as c_char
            && output + 2 < candidate.len()
        {
            candidate[output] = unsafe { directory.read() };
            directory = unsafe { directory.add(1) };
            output += 1;
        }
        if output != 0 {
            candidate[output] = b'/' as c_char;
            output += 1;
        }
        let mut name = file;
        while unsafe { name.read() } != 0 && output + 1 < candidate.len() {
            candidate[output] = unsafe { name.read() };
            name = unsafe { name.add(1) };
            output += 1;
        }
        candidate[output] = 0;
        unsafe {
            syscall3(
                SYS_EXECVE,
                candidate.as_ptr() as usize,
                arguments as usize,
                environ as usize,
            )
        };
        if unsafe { directory.read() } == 0 {
            break;
        }
        directory = unsafe { directory.add(1) };
    }
    convert_syscall_result(-2) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_WAIT4,
            pid as usize,
            status as usize,
            options as usize,
            0,
            0,
            0,
        )
    }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn waitid(kind: c_int, id: u32, info: *mut c_void, options: c_int) -> c_int {
    convert_syscall_result(unsafe {
        syscall6(
            SYS_WAITID,
            kind as usize,
            id as usize,
            info as usize,
            options as usize,
            0,
            0,
        )
    }) as c_int
}

macro_rules! one_argument_call {
    ($name:ident, $number:ident, $kind:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(value: $kind) -> c_int {
            convert_syscall_result(unsafe { syscall1($number, value as usize) }) as c_int
        }
    };
}

one_argument_call!(chdir, SYS_CHDIR, *const c_char);
one_argument_call!(chroot, SYS_CHROOT, *const c_char);
one_argument_call!(setgid, SYS_SETGID, u32);
one_argument_call!(setuid, SYS_SETUID, u32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setgroups(count: usize, groups: *const u32) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_SETGROUPS, count, groups as usize, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setpgid(pid: c_int, group: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_SETPGID, pid as usize, group as usize, 0) })
        as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn setsid() -> c_int {
    convert_syscall_result(unsafe { syscall1(SYS_SETSID, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pipe2(files: *mut c_int, flags: c_int) -> c_int {
    convert_syscall_result(unsafe { syscall3(SYS_PIPE2, files as usize, flags as usize, 0) })
        as c_int
}

macro_rules! posix_spawn_stub {
    ($($name:ident($($argument:ident: $kind:ty),* $(,)?)),* $(,)?) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($argument: $kind),*) -> c_int {
                $(let _ = $argument;)*
                38
            }
        )*
    };
}

posix_spawn_stub!(
    posix_spawnattr_init(value: *mut c_void),
    posix_spawnattr_destroy(value: *mut c_void),
    posix_spawnattr_setflags(value: *mut c_void, flags: i16),
    posix_spawnattr_setpgroup(value: *mut c_void, group: c_int),
    posix_spawnattr_setsigdefault(value: *mut c_void, signals: *const c_void),
    posix_spawn_file_actions_init(value: *mut c_void),
    posix_spawn_file_actions_destroy(value: *mut c_void),
    posix_spawn_file_actions_adddup2(value: *mut c_void, fd: c_int, target: c_int),
    posix_spawnp(
        pid: *mut c_int,
        path: *const c_char,
        actions: *const c_void,
        attributes: *const c_void,
        arguments: *const *const c_char,
        environment: *const *const c_char,
    ),
);

#[unsafe(no_mangle)]
pub extern "C" fn gnu_get_libc_version() -> *const u8 {
    c"0.0".as_ptr().cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpwuid_r(
    _uid: u32,
    _password: *mut c_void,
    _buffer: *mut c_char,
    _length: usize,
    result: *mut *mut c_void,
) -> c_int {
    if !result.is_null() {
        unsafe { result.write(core::ptr::null_mut()) };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pause() -> c_int {
    unsafe { pause_impl() }
}
