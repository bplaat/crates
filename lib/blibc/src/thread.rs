use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, Ordering};

use crate::alloc::{free, malloc};
use crate::syscall::{syscall1, syscall3, syscall6};

cfg_select! {
    target_arch = "x86_64" => {
        const SYS_FUTEX: usize = 202;
        const SYS_GETTID: usize = 186;
        const SYS_SCHED_GETAFFINITY: usize = 204;
        const SYS_SCHED_YIELD: usize = 24;

        core::arch::global_asm!(
            r#"
            .global blibc_clone_thread
            .type blibc_clone_thread,@function
        blibc_clone_thread:
            mov r10, rcx
            mov r9, r8
            xor r8d, r8d
            mov eax, 56
            syscall
            test rax, rax
            jnz 1f
            mov rdi, r9
            call blibc_thread_start
            xor edi, edi
            mov eax, 60
            syscall
            hlt
        1:
            ret
        .size blibc_clone_thread, .-blibc_clone_thread
        "#
        );
    }
    target_arch = "aarch64" => {
        const SYS_FUTEX: usize = 98;
        const SYS_GETTID: usize = 178;
        const SYS_SCHED_GETAFFINITY: usize = 123;
        const SYS_SCHED_YIELD: usize = 124;

        core::arch::global_asm!(
            r#"
            .global blibc_clone_thread
            .type blibc_clone_thread,%function
        blibc_clone_thread:
            mov x6, x4
            mov x4, x3
            mov x3, xzr
            mov x8, #220
            svc #0
            cbnz x0, 1f
            mov x0, x6
            bl blibc_thread_start
            mov x0, xzr
            mov x8, #93
            svc #0
            brk #0
        1:
            ret
        .size blibc_clone_thread, .-blibc_clone_thread
        "#
        );
    }
    _ => {
        compile_error!("unsupported target architecture");
    }
}
const MAX_KEYS: usize = 128;
const MAX_TLS_THREADS: usize = 1024;
const STACK_SIZE: usize = 2 * 1024 * 1024;
const CLONE_FLAGS: usize = 0x350f00;
const FUTEX_WAIT: usize = 0;
const FUTEX_WAKE: usize = 1;
const FUTEX_PRIVATE: usize = 128;

static TLS_LOCKED: AtomicBool = AtomicBool::new(false);
static mut TLS_KEYS: [bool; MAX_KEYS] = [false; MAX_KEYS];
static mut TLS_DESTRUCTORS: [Option<unsafe extern "C" fn(*mut c_void)>; MAX_KEYS] =
    [None; MAX_KEYS];
static mut TLS_THREADS: [TlsThread; MAX_TLS_THREADS] = [const {
    TlsThread {
        tid: 0,
        errno: 0,
        values: [ptr::null_mut(); MAX_KEYS],
    }
}; MAX_TLS_THREADS];

struct TlsThread {
    tid: i32,
    errno: c_int,
    values: [*mut c_void; MAX_KEYS],
}

struct TlsGuard;

impl TlsGuard {
    fn lock() -> Self {
        while TLS_LOCKED
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self
    }
}

impl Drop for TlsGuard {
    fn drop(&mut self) {
        TLS_LOCKED.store(false, Ordering::Release);
    }
}

fn current_tid() -> i32 {
    unsafe { syscall1(SYS_GETTID, 0) as i32 }
}

pub(crate) fn errno_location() -> *mut c_int {
    static mut FALLBACK_ERRNO: c_int = 0;

    let _guard = TlsGuard::lock();
    let thread = unsafe { tls_slot(current_tid(), true) };
    if thread.is_null() {
        ptr::addr_of_mut!(FALLBACK_ERRNO)
    } else {
        unsafe { ptr::addr_of_mut!((*thread).errno) }
    }
}

unsafe fn tls_slot(tid: i32, create: bool) -> *mut TlsThread {
    let threads = ptr::addr_of_mut!(TLS_THREADS).cast::<TlsThread>();
    let mut empty: *mut TlsThread = ptr::null_mut();
    for index in 0..MAX_TLS_THREADS {
        let thread = unsafe { threads.add(index) };
        let slot_tid = unsafe { (*thread).tid };
        if slot_tid == tid {
            return thread;
        }
        if slot_tid == 0 && empty.is_null() {
            empty = thread;
        }
    }
    if create && !empty.is_null() {
        unsafe { (*empty).tid = tid };
        return empty;
    }
    ptr::null_mut()
}

unsafe fn destroy_tls() {
    let tid = current_tid();
    for _ in 0..4 {
        let mut called_destructor = false;
        for key in 0..MAX_KEYS {
            let (value, destructor) = {
                let _guard = TlsGuard::lock();
                let thread = unsafe { tls_slot(tid, false) };
                if thread.is_null() {
                    return;
                }
                let value = unsafe { (*thread).values[key] };
                unsafe { (*thread).values[key] = ptr::null_mut() };
                (value, unsafe { TLS_DESTRUCTORS[key] })
            };
            if !value.is_null()
                && let Some(destructor) = destructor
            {
                called_destructor = true;
                unsafe { destructor(value) };
            }
        }
        if !called_destructor {
            break;
        }
    }
    let _guard = TlsGuard::lock();
    let thread = unsafe { tls_slot(tid, false) };
    if !thread.is_null() {
        unsafe {
            (*thread).errno = 0;
            (*thread).values.fill(ptr::null_mut());
            (*thread).tid = 0;
        }
    }
}

#[repr(C)]
struct Thread {
    tid: AtomicI32,
    result: AtomicPtr<c_void>,
    stack: *mut c_void,
    start: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    argument: *mut c_void,
}

unsafe extern "C" {
    fn blibc_clone_thread(
        flags: usize,
        stack: *mut c_void,
        parent_tid: *mut AtomicI32,
        child_tid: *mut AtomicI32,
        thread: *mut Thread,
    ) -> isize;
}

unsafe fn futex(word: *mut AtomicI32, operation: usize, value: i32) -> isize {
    unsafe { syscall6(SYS_FUTEX, word as usize, operation, value as usize, 0, 0, 0) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn blibc_thread_start(thread: *mut Thread) {
    let result = unsafe { ((*thread).start)((*thread).argument) };
    unsafe { (*thread).result.store(result, Ordering::Release) };
    unsafe { destroy_tls() };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_key_create(
    key: *mut u32,
    destructor: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    let _guard = TlsGuard::lock();
    for index in 0..MAX_KEYS {
        if !unsafe { TLS_KEYS[index] } {
            unsafe {
                TLS_KEYS[index] = true;
                TLS_DESTRUCTORS[index] = destructor;
                key.write(index as u32);
            }
            return 0;
        }
    }
    11
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_key_delete(key: u32) -> c_int {
    if key as usize >= MAX_KEYS {
        return 22;
    }
    let _guard = TlsGuard::lock();
    if !unsafe { TLS_KEYS[key as usize] } {
        return 22;
    }
    unsafe {
        TLS_KEYS[key as usize] = false;
        TLS_DESTRUCTORS[key as usize] = None;
        let threads = ptr::addr_of_mut!(TLS_THREADS).cast::<TlsThread>();
        for index in 0..MAX_TLS_THREADS {
            (*threads.add(index)).values[key as usize] = ptr::null_mut();
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_getspecific(key: u32) -> *mut c_void {
    if key as usize >= MAX_KEYS {
        return ptr::null_mut();
    }
    let _guard = TlsGuard::lock();
    if !unsafe { TLS_KEYS[key as usize] } {
        return ptr::null_mut();
    }
    let thread = unsafe { tls_slot(current_tid(), false) };
    if thread.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*thread).values[key as usize] }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_setspecific(key: u32, value: *const c_void) -> c_int {
    if key as usize >= MAX_KEYS {
        return 22;
    }
    let _guard = TlsGuard::lock();
    if !unsafe { TLS_KEYS[key as usize] } {
        return 22;
    }
    let thread = unsafe { tls_slot(current_tid(), true) };
    if thread.is_null() {
        return 11;
    }
    unsafe { (*thread).values[key as usize] = value.cast_mut() };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_self() -> usize {
    current_tid() as usize
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_getattr_np(_thread: usize, _attributes: *mut c_void) -> c_int {
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getstack(
    _attributes: *const c_void,
    _address: *mut *mut c_void,
    _size: *mut usize,
) -> c_int {
    22
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getguardsize(
    _attributes: *const c_void,
    _size: *mut usize,
) -> c_int {
    22
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_destroy(_attributes: *mut c_void) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_init(_attributes: *mut c_void) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setstacksize(
    _attributes: *mut c_void,
    _size: usize,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_create(
    thread: *mut usize,
    _attributes: *const c_void,
    start: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    argument: *mut c_void,
) -> c_int {
    let stack = unsafe { malloc(STACK_SIZE) };
    if stack.is_null() {
        return 11;
    }
    let context = unsafe { malloc(core::mem::size_of::<Thread>()) }.cast::<Thread>();
    if context.is_null() {
        unsafe { free(stack) };
        return 11;
    }
    unsafe {
        context.write(Thread {
            tid: AtomicI32::new(0),
            result: AtomicPtr::new(ptr::null_mut()),
            stack,
            start,
            argument,
        });
        let stack_top = stack.cast::<u8>().add(STACK_SIZE).cast();
        let tid_pointer = ptr::addr_of_mut!((*context).tid);
        let result = blibc_clone_thread(CLONE_FLAGS, stack_top, tid_pointer, tid_pointer, context);
        if result < 0 {
            free(context.cast());
            free(stack);
            return -result as c_int;
        }
        thread.write(context as usize);
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_join(thread: usize, result: *mut *mut c_void) -> c_int {
    let context = thread as *mut Thread;
    loop {
        let tid = unsafe { (*context).tid.load(Ordering::Acquire) };
        if tid == 0 {
            break;
        }
        unsafe { futex(ptr::addr_of_mut!((*context).tid), FUTEX_WAIT, tid) };
    }
    if !result.is_null() {
        unsafe { result.write((*context).result.load(Ordering::Acquire)) };
    }
    unsafe {
        free((*context).stack);
        free(context.cast());
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_detach(_thread: usize) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_setname_np(_thread: usize, _name: *const c_char) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sched_yield() -> c_int {
    crate::errno::convert_syscall_result(unsafe { syscall1(SYS_SCHED_YIELD, 0) }) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sched_getaffinity(pid: c_int, size: usize, mask: *mut c_void) -> c_int {
    crate::errno::convert_syscall_result(unsafe {
        syscall3(SYS_SCHED_GETAFFINITY, pid as usize, size, mask as usize)
    }) as c_int
}

unsafe fn mutex_state(mutex: *mut c_void) -> *mut AtomicI32 {
    mutex.cast()
}

unsafe fn mutex_count(mutex: *mut c_void) -> *mut u32 {
    unsafe { mutex.cast::<u8>().add(4).cast() }
}

unsafe fn mutex_owner(mutex: *mut c_void) -> *mut AtomicI32 {
    unsafe { mutex.cast::<u8>().add(8).cast() }
}

unsafe fn mutex_kind(mutex: *mut c_void) -> *mut c_int {
    unsafe { mutex.cast::<u8>().add(16).cast() }
}

unsafe fn finish_mutex_lock(mutex: *mut c_void, tid: i32) {
    unsafe {
        (*mutex_owner(mutex)).store(tid, Ordering::Relaxed);
        mutex_count(mutex).write(1);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut c_void,
    attributes: *const c_void,
) -> c_int {
    let kind = if attributes.is_null() {
        0
    } else {
        unsafe { attributes.cast::<c_int>().read() }
    };
    unsafe {
        mutex_state(mutex).write(AtomicI32::new(0));
        mutex_count(mutex).write(0);
        mutex_owner(mutex).write(AtomicI32::new(0));
        mutex_kind(mutex).write(kind);
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut c_void) -> c_int {
    if unsafe { (*mutex_state(mutex)).load(Ordering::Relaxed) } == 0 {
        0
    } else {
        16
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut c_void) -> c_int {
    let tid = current_tid();
    if unsafe { mutex_kind(mutex).read() } & 3 == 1
        && unsafe { (*mutex_owner(mutex)).load(Ordering::Relaxed) } == tid
    {
        let Some(count) = unsafe { mutex_count(mutex).read() }.checked_add(1) else {
            return 11;
        };
        unsafe { mutex_count(mutex).write(count) };
        return 0;
    }
    if unsafe { mutex_kind(mutex).read() } & 3 == 2
        && unsafe { (*mutex_owner(mutex)).load(Ordering::Relaxed) } == tid
    {
        return 35;
    }

    let state = unsafe { &*mutex_state(mutex) };
    if state
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        unsafe { finish_mutex_lock(mutex, tid) };
        return 0;
    }
    loop {
        if state.swap(2, Ordering::Acquire) == 0 {
            unsafe { finish_mutex_lock(mutex, tid) };
            return 0;
        }
        unsafe { futex(mutex_state(mutex), FUTEX_WAIT | FUTEX_PRIVATE, 2) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut c_void) -> c_int {
    let tid = current_tid();
    if unsafe { mutex_kind(mutex).read() } & 3 == 1
        && unsafe { (*mutex_owner(mutex)).load(Ordering::Relaxed) } == tid
    {
        let Some(count) = unsafe { mutex_count(mutex).read() }.checked_add(1) else {
            return 11;
        };
        unsafe { mutex_count(mutex).write(count) };
        return 0;
    }
    if unsafe { (*mutex_state(mutex)).compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed) }
        .is_err()
    {
        return 16;
    }
    unsafe { finish_mutex_lock(mutex, tid) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut c_void) -> c_int {
    if matches!(unsafe { mutex_kind(mutex).read() } & 3, 1 | 2) {
        if unsafe { (*mutex_owner(mutex)).load(Ordering::Relaxed) } != current_tid() {
            return 1;
        }
        let count = unsafe { mutex_count(mutex).read() };
        if count > 1 {
            unsafe { mutex_count(mutex).write(count - 1) };
            return 0;
        }
    }
    unsafe {
        (*mutex_owner(mutex)).store(0, Ordering::Relaxed);
        mutex_count(mutex).write(0);
    }
    if unsafe { (*mutex_state(mutex)).swap(0, Ordering::Release) } == 2 {
        unsafe { futex(mutex_state(mutex), FUTEX_WAKE | FUTEX_PRIVATE, 1) };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_init(attributes: *mut c_void) -> c_int {
    unsafe { attributes.cast::<c_int>().write(0) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_destroy(_attributes: *mut c_void) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_settype(attributes: *mut c_void, kind: c_int) -> c_int {
    if !(0..=3).contains(&kind) {
        return 22;
    }
    unsafe { attributes.cast::<c_int>().write(kind) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    const SC_CLK_TCK: c_int = 2;
    const SC_PAGESIZE: c_int = 30;
    const SC_NPROCESSORS_CONF: c_int = 83;
    const SC_NPROCESSORS_ONLN: c_int = 84;

    match name {
        SC_CLK_TCK => 100,
        SC_PAGESIZE => 4096,
        SC_NPROCESSORS_CONF | SC_NPROCESSORS_ONLN => {
            let mut mask = [0usize; 16];
            let result = unsafe {
                syscall3(
                    SYS_SCHED_GETAFFINITY,
                    0,
                    core::mem::size_of_val(&mask),
                    mask.as_mut_ptr() as usize,
                )
            };
            if result < 0 {
                crate::errno::set_errno(-result as c_int);
                -1
            } else {
                mask.iter().map(|word| word.count_ones() as c_long).sum()
            }
        }
        _ => {
            unsafe { crate::errno::__errno_location().write(22) };
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_trylock,
        pthread_mutex_unlock, pthread_mutexattr_init, pthread_mutexattr_settype,
    };

    #[test]
    fn normal_and_recursive_mutexes_work() {
        let mut mutex = [0_u64; 5];
        let mutex_pointer = mutex.as_mut_ptr().cast();
        unsafe {
            assert_eq!(pthread_mutex_init(mutex_pointer, core::ptr::null()), 0);
            assert_eq!(pthread_mutex_lock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_trylock(mutex_pointer), 16);
            assert_eq!(pthread_mutex_unlock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_destroy(mutex_pointer), 0);
        }

        let mut attributes = [0_u64; 1];
        unsafe {
            assert_eq!(pthread_mutexattr_init(attributes.as_mut_ptr().cast()), 0);
            assert_eq!(
                pthread_mutexattr_settype(attributes.as_mut_ptr().cast(), 1),
                0
            );
            assert_eq!(
                pthread_mutex_init(mutex_pointer, attributes.as_ptr().cast()),
                0
            );
            assert_eq!(pthread_mutex_lock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_lock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_unlock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_unlock(mutex_pointer), 0);
            assert_eq!(pthread_mutex_destroy(mutex_pointer), 0);
        }
    }
}
