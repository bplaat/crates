use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    destination: *mut c_void,
    source: *const c_void,
    count: usize,
) -> *mut c_void {
    for index in 0..count {
        unsafe {
            ptr::write_volatile(
                destination.cast::<u8>().add(index),
                ptr::read_volatile(source.cast::<u8>().add(index)),
            );
        }
    }
    destination
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(
    destination: *mut c_void,
    source: *const c_void,
    count: usize,
) -> *mut c_void {
    if (destination as usize) < source as usize {
        unsafe { memcpy(destination, source, count) };
    } else {
        for index in (0..count).rev() {
            unsafe {
                ptr::write_volatile(
                    destination.cast::<u8>().add(index),
                    ptr::read_volatile(source.cast::<u8>().add(index)),
                );
            }
        }
    }
    destination
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
    destination: *mut c_void,
    value: c_int,
    count: usize,
) -> *mut c_void {
    for index in 0..count {
        unsafe { ptr::write_volatile(destination.cast::<u8>().add(index), value as u8) };
    }
    destination
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(left: *const c_void, right: *const c_void, count: usize) -> c_int {
    for index in 0..count {
        let difference = unsafe {
            *left.cast::<u8>().add(index) as c_int - *right.cast::<u8>().add(index) as c_int
        };
        if difference != 0 {
            return difference;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(left: *const c_void, right: *const c_void, count: usize) -> c_int {
    unsafe { memcmp(left, right, count) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(value: *const c_char) -> usize {
    let mut length = 0;
    while unsafe { *value.add(length) } != 0 {
        length += 1;
    }
    length
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memchr(value: *const c_void, byte: c_int, count: usize) -> *mut c_void {
    for index in 0..count {
        let current = unsafe { value.cast::<u8>().add(index).read() };
        if current == byte as u8 {
            return unsafe { value.cast::<u8>().add(index).cast_mut().cast() };
        }
    }
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(left: *const c_char, right: *const c_char) -> c_int {
    unsafe { strncmp(left, right, usize::MAX) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(left: *const c_char, right: *const c_char, count: usize) -> c_int {
    for index in 0..count {
        let left_byte = unsafe { left.add(index).read() } as u8;
        let right_byte = unsafe { right.add(index).read() } as u8;
        if left_byte != right_byte || left_byte == 0 {
            return left_byte as c_int - right_byte as c_int;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(value: *const c_char, byte: c_int) -> *mut c_char {
    let mut current = value;
    loop {
        let current_byte = unsafe { current.read() } as u8;
        if current_byte == byte as u8 {
            return current.cast_mut();
        }
        if current_byte == 0 {
            return ptr::null_mut();
        }
        current = unsafe { current.add(1) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strrchr(value: *const c_char, byte: c_int) -> *mut c_char {
    let mut current = value;
    let mut found = ptr::null_mut();
    loop {
        let current_byte = unsafe { current.read() } as u8;
        if current_byte == byte as u8 {
            found = current.cast_mut();
        }
        if current_byte == 0 {
            return found;
        }
        current = unsafe { current.add(1) };
    }
}

unsafe fn byte_in_set(byte: u8, set: *const c_char) -> bool {
    let mut current = set;
    while unsafe { current.read() } != 0 {
        if unsafe { current.read() } as u8 == byte {
            return true;
        }
        current = unsafe { current.add(1) };
    }
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strspn(value: *const c_char, accepted: *const c_char) -> usize {
    let mut length = 0;
    while unsafe { value.add(length).read() } != 0
        && unsafe { byte_in_set(value.add(length).read() as u8, accepted) }
    {
        length += 1;
    }
    length
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcspn(value: *const c_char, rejected: *const c_char) -> usize {
    let mut length = 0;
    while unsafe { value.add(length).read() } != 0
        && !unsafe { byte_in_set(value.add(length).read() as u8, rejected) }
    {
        length += 1;
    }
    length
}
