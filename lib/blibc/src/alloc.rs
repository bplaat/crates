use core::ffi::{c_int, c_void};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use dlmalloc::Dlmalloc;

use crate::string::memset;

static LOCKED: AtomicBool = AtomicBool::new(false);
static mut ALLOCATOR: Dlmalloc = Dlmalloc::new();

struct AllocatorGuard;

impl AllocatorGuard {
    fn lock() -> Self {
        while LOCKED
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self
    }

    fn allocator(&mut self) -> *mut Dlmalloc {
        ptr::addr_of_mut!(ALLOCATOR)
    }
}

impl Drop for AllocatorGuard {
    fn drop(&mut self) {
        LOCKED.store(false, Ordering::Release);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let mut guard = AllocatorGuard::lock();
    let allocation: *mut c_void = unsafe { (*guard.allocator()).c_malloc(size).cast() };
    if allocation.is_null() {
        crate::errno::set_errno(12);
    }
    allocation
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    let Some(total) = count.checked_mul(size) else {
        crate::errno::set_errno(12);
        return ptr::null_mut();
    };
    let allocation = unsafe { malloc(total) };
    if !allocation.is_null() {
        unsafe { memset(allocation, 0, total) };
    }
    allocation
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(allocation: *mut c_void, size: usize) -> *mut c_void {
    let mut guard = AllocatorGuard::lock();
    let resized: *mut c_void = unsafe {
        (*guard.allocator())
            .c_realloc(allocation.cast(), size)
            .cast()
    };
    if resized.is_null() && size != 0 {
        crate::errno::set_errno(12);
    }
    resized
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(allocation: *mut c_void) {
    let mut guard = AllocatorGuard::lock();
    unsafe { (*guard.allocator()).c_free(allocation.cast()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_memalign(
    output: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    if !alignment.is_power_of_two() || alignment < core::mem::size_of::<usize>() {
        return 22;
    }
    let mut guard = AllocatorGuard::lock();
    let allocation = unsafe { (*guard.allocator()).c_memalign(alignment, size) };
    if allocation.is_null() {
        return 12;
    }
    unsafe { output.write(allocation.cast()) };
    0
}
