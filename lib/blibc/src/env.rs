use core::ffi::{c_char, c_int};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::alloc::{free, malloc};
use crate::errno::{convert_syscall_result, set_errno};
use crate::io::{close, open64, readlink};
use crate::string::{memchr, memcpy, strlen, strncmp};
use crate::syscall::syscall3;

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_GETCWD: usize = 79;
    }
    target_arch = "aarch64" => {
        const SYS_GETCWD: usize = 17;
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}

unsafe extern "C" {
    static mut environ: *mut *mut c_char;
}

const MAX_ENVIRONMENT_ENTRIES: usize = 256;

static ENVIRONMENT_LOCKED: AtomicBool = AtomicBool::new(false);
static mut ENVIRONMENT: [*mut c_char; MAX_ENVIRONMENT_ENTRIES + 1] =
    [ptr::null_mut(); MAX_ENVIRONMENT_ENTRIES + 1];
static mut ENVIRONMENT_OWNED: [bool; MAX_ENVIRONMENT_ENTRIES] = [false; MAX_ENVIRONMENT_ENTRIES];
static mut ENVIRONMENT_INITIALIZED: bool = false;
static mut AUXILIARY: *const usize = ptr::null();

struct EnvironmentGuard;

impl EnvironmentGuard {
    fn lock() -> Self {
        while ENVIRONMENT_LOCKED
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        ENVIRONMENT_LOCKED.store(false, Ordering::Release);
    }
}

unsafe fn initialize_environment() -> bool {
    if unsafe { ENVIRONMENT_INITIALIZED } {
        return true;
    }
    let source = unsafe { environ };
    if source.is_null() {
        unsafe {
            environ = ptr::addr_of_mut!(ENVIRONMENT).cast::<*mut c_char>();
            ENVIRONMENT_INITIALIZED = true;
        }
        return true;
    }
    let mut count = 0;
    while !unsafe { source.add(count).read() }.is_null() {
        if count == MAX_ENVIRONMENT_ENTRIES {
            set_errno(12);
            return false;
        }
        unsafe { ENVIRONMENT[count] = source.add(count).read() };
        count += 1;
    }
    unsafe {
        AUXILIARY = source.add(count + 1).cast();
        environ = ptr::addr_of_mut!(ENVIRONMENT).cast::<*mut c_char>();
        ENVIRONMENT_INITIALIZED = true;
    }
    true
}

unsafe fn find_environment_entry(name: *const c_char, name_length: usize) -> Option<usize> {
    for index in 0..MAX_ENVIRONMENT_ENTRIES {
        let entry = unsafe { ENVIRONMENT[index] };
        if entry.is_null() {
            return None;
        }
        if unsafe { entry.add(name_length).read() } == b'=' as c_char
            && unsafe { strncmp(entry, name, name_length) } == 0
        {
            return Some(index);
        }
    }
    None
}

unsafe fn environment_length() -> usize {
    let mut length = 0;
    while length < MAX_ENVIRONMENT_ENTRIES && !unsafe { ENVIRONMENT[length] }.is_null() {
        length += 1;
    }
    length
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    let name_length = unsafe { strlen(name) };
    if name_length == 0 || !unsafe { memchr(name.cast(), b'=' as c_int, name_length) }.is_null() {
        return ptr::null_mut();
    }
    let _guard = EnvironmentGuard::lock();
    if !unsafe { initialize_environment() } {
        return ptr::null_mut();
    }
    let mut current = unsafe { environ };
    while !unsafe { current.read() }.is_null() {
        let entry = unsafe { current.read() };
        let separator = unsafe { entry.add(name_length) };
        if unsafe { separator.read() } == b'=' as c_char
            && unsafe { strncmp(entry, name, name_length) } == 0
        {
            return unsafe { separator.add(1) };
        }
        current = unsafe { current.add(1) };
    }
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcwd(buffer: *mut c_char, size: usize) -> *mut c_char {
    if convert_syscall_result(unsafe { syscall3(SYS_GETCWD, buffer as usize, size, 0) }) < 0 {
        ptr::null_mut()
    } else {
        buffer
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realpath(path: *const c_char, output: *mut c_char) -> *mut c_char {
    const O_CLOEXEC: c_int = 0x80000;
    const O_PATH: c_int = 0x200000;
    const PATH_MAX: usize = 4096;

    let fd = unsafe { open64(path, O_PATH | O_CLOEXEC, 0) };
    if fd < 0 {
        return ptr::null_mut();
    }
    let allocated = output.is_null();
    let output = if allocated {
        unsafe { malloc(PATH_MAX) }.cast::<c_char>()
    } else {
        output
    };
    if output.is_null() {
        unsafe { close(fd) };
        return ptr::null_mut();
    }

    let mut link = *b"/proc/self/fd/0000000000\0";
    let mut number = fd as u32;
    let mut position = link.len() - 2;
    loop {
        link[position] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
        position -= 1;
    }
    let prefix_length = b"/proc/self/fd/".len();
    let digits_end = link.len() - 1;
    link.copy_within(position..digits_end, prefix_length);
    link[prefix_length + digits_end - position] = 0;
    let length = unsafe { readlink(link.as_ptr().cast(), output, PATH_MAX - 1) };
    unsafe { close(fd) };
    if length < 0 {
        if allocated {
            unsafe { free(output.cast()) };
        }
        return ptr::null_mut();
    }
    unsafe { output.add(length as usize).write(0) };
    output
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setenv(
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    let name_length = unsafe { strlen(name) };
    if name_length == 0 || !unsafe { memchr(name.cast(), b'=' as c_int, name_length) }.is_null() {
        set_errno(22);
        return -1;
    }
    let _guard = EnvironmentGuard::lock();
    if !unsafe { initialize_environment() } {
        return -1;
    }
    let existing = unsafe { find_environment_entry(name, name_length) };
    if existing.is_some() && overwrite == 0 {
        return 0;
    }
    let value_length = unsafe { strlen(value) };
    let Some(length) = name_length
        .checked_add(value_length)
        .and_then(|length| length.checked_add(2))
    else {
        set_errno(12);
        return -1;
    };
    let entry = unsafe { malloc(length) }.cast::<c_char>();
    if entry.is_null() {
        return -1;
    }
    unsafe {
        memcpy(entry.cast(), name.cast(), name_length);
        entry.add(name_length).write(b'=' as c_char);
        memcpy(
            entry.add(name_length + 1).cast(),
            value.cast(),
            value_length + 1,
        );
    }
    let index = match existing {
        Some(index) => index,
        None => unsafe { environment_length() },
    };
    if index == MAX_ENVIRONMENT_ENTRIES {
        unsafe { free(entry.cast()) };
        set_errno(12);
        return -1;
    }
    let old_entry = unsafe { ENVIRONMENT[index] };
    let old_owned = unsafe { ENVIRONMENT_OWNED[index] };
    unsafe {
        ENVIRONMENT[index] = entry;
        ENVIRONMENT_OWNED[index] = true;
        if existing.is_none() {
            ENVIRONMENT[index + 1] = ptr::null_mut();
        }
    }
    if old_owned {
        unsafe { free(old_entry.cast()) };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unsetenv(name: *const c_char) -> c_int {
    let name_length = unsafe { strlen(name) };
    if name_length == 0 || !unsafe { memchr(name.cast(), b'=' as c_int, name_length) }.is_null() {
        set_errno(22);
        return -1;
    }
    let _guard = EnvironmentGuard::lock();
    if !unsafe { initialize_environment() } {
        return -1;
    }
    let Some(index) = (unsafe { find_environment_entry(name, name_length) }) else {
        return 0;
    };
    let old_entry = unsafe { ENVIRONMENT[index] };
    let old_owned = unsafe { ENVIRONMENT_OWNED[index] };
    let length = unsafe { environment_length() };
    for position in index..length {
        unsafe {
            ENVIRONMENT[position] = ENVIRONMENT[position + 1];
            ENVIRONMENT_OWNED[position] = if position + 1 < MAX_ENVIRONMENT_ENTRIES {
                ENVIRONMENT_OWNED[position + 1]
            } else {
                false
            };
        }
    }
    if length != 0 {
        unsafe { ENVIRONMENT_OWNED[length - 1] = false };
    }
    if old_owned {
        unsafe { free(old_entry.cast()) };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getauxval(kind: usize) -> usize {
    let _guard = EnvironmentGuard::lock();
    if !unsafe { initialize_environment() } {
        return 0;
    }
    let mut auxiliary = unsafe { AUXILIARY };
    if auxiliary.is_null() {
        set_errno(2);
        return 0;
    }
    while unsafe { auxiliary.read() } != 0 {
        if unsafe { auxiliary.read() } == kind {
            return unsafe { auxiliary.add(1).read() };
        }
        auxiliary = unsafe { auxiliary.add(2) };
    }
    set_errno(2);
    0
}

#[cfg(test)]
mod tests {
    use super::{getenv, setenv, unsetenv};
    use crate::string::strcmp;

    #[test]
    fn environment_can_be_updated() {
        unsafe {
            assert_eq!(unsetenv(c"BLIBC_TEST_VALUE".as_ptr()), 0);
            assert!(getenv(c"BLIBC_TEST_VALUE".as_ptr()).is_null());
            assert_eq!(
                setenv(c"BLIBC_TEST_VALUE".as_ptr(), c"first".as_ptr(), 1),
                0
            );
            assert_eq!(
                strcmp(getenv(c"BLIBC_TEST_VALUE".as_ptr()), c"first".as_ptr()),
                0
            );
            assert_eq!(
                setenv(c"BLIBC_TEST_VALUE".as_ptr(), c"second".as_ptr(), 0),
                0
            );
            assert_eq!(
                strcmp(getenv(c"BLIBC_TEST_VALUE".as_ptr()), c"first".as_ptr()),
                0
            );
            assert_eq!(
                setenv(c"BLIBC_TEST_VALUE".as_ptr(), c"second".as_ptr(), 1),
                0
            );
            assert_eq!(
                strcmp(getenv(c"BLIBC_TEST_VALUE".as_ptr()), c"second".as_ptr()),
                0
            );
            assert_eq!(unsetenv(c"BLIBC_TEST_VALUE".as_ptr()), 0);
            assert!(getenv(c"BLIBC_TEST_VALUE".as_ptr()).is_null());
        }
    }
}
