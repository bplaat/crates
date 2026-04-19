/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [block2](https://crates.io/crates/block2) crate

#![cfg(target_vendor = "apple")]
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use objc2::encode::{Encode, Encoding};

#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    static _NSConcreteMallocBlock: *const c_void;
}

#[repr(C)]
struct BlockDescriptor {
    reserved: u64,
    size: u64,
}

const BLOCK_HAS_DESCRIPTOR: i32 = 1 << 25;

static BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: size_of::<BlockDescriptor>() as u64,
};

/// An Objective-C block. `F` is the function signature (e.g. `dyn Fn(i64)`).
/// This type is `repr(C)` matching the ObjC block ABI, so `&Block<F>` is a thin pointer.
#[repr(C)]
pub struct Block<F: ?Sized> {
    _isa: *const c_void,
    _flags: i32,
    _reserved: i32,
    _invoke: *const c_void,
    _descriptor: *const BlockDescriptor,
    _marker: PhantomData<*const F>,
}

// SAFETY: Sending a `Block<F>` to another thread is safe iff the captured closure F is Send.
unsafe impl<F: Send + ?Sized> Send for Block<F> {}
// SAFETY: Sharing `&Block<F>` across threads is safe iff the captured closure F is Sync.
unsafe impl<F: Sync + ?Sized> Sync for Block<F> {}

// A block argument is encoded as `@?` (an object which is a block)
// SAFETY: A block pointer always has encoding `@?` in the ObjC type system.
unsafe impl<F: ?Sized> Encode for &Block<F> {
    const ENCODING: Encoding = Encoding::Block;
}

macro_rules! impl_block_call {
    ($($t:ident: $a:ident),*) => {
        impl<$($t: 'static + Copy),*> Block<dyn Fn($($t,)*)> {
            /// Call this block with the given arguments.
            pub fn call(&self, ($($a,)*): ($($t,)*)) {
                // SAFETY: `_invoke` was stored by `RcBlock::make` as an `extern "C"` fn with
                // exactly this signature. The block ABI stores all invoke pointers as opaque
                // `*const c_void`, so the transmute is required. `self` is a valid shared
                // reference that remains valid for the duration of this call.
                unsafe {
                    let invoke: unsafe extern "C" fn(*const c_void $(, $t)*) =
                        std::mem::transmute(self._invoke);
                    invoke(self as *const Self as *const c_void $(, $a)*);
                }
            }
        }
    };
}
impl_block_call!();
impl_block_call!(A: a);
impl_block_call!(A: a, B: b);
impl_block_call!(A: a, B: b, C: c);
impl_block_call!(A: a, B: b, C: c, D: d);

/// Inner heap layout for `RcBlock`: ObjC block header immediately followed by the closure.
#[repr(C)]
struct RcBlockInner<F> {
    block: Block<F>,
    closure: F,
}

/// A heap-allocated ObjC block wrapping a Rust closure.
pub struct RcBlock<F> {
    inner: NonNull<RcBlockInner<F>>,
}

// SAFETY: Sending an `RcBlock<F>` to another thread is safe iff F is Send.
unsafe impl<F: Send> Send for RcBlock<F> {}
// SAFETY: Sharing `&RcBlock<F>` across threads is safe iff F is Sync.
unsafe impl<F: Sync> Sync for RcBlock<F> {}

impl<F> RcBlock<F> {
    fn make(closure: F, invoke: *const c_void) -> Self {
        let inner = Box::new(RcBlockInner {
            block: Block {
                // SAFETY: `_NSConcreteMallocBlock` is a valid extern static exported by
                // libSystem; it is always initialised before any Rust code runs.
                _isa: unsafe { _NSConcreteMallocBlock },
                _flags: BLOCK_HAS_DESCRIPTOR,
                _reserved: 0,
                _invoke: invoke,
                _descriptor: &BLOCK_DESCRIPTOR,
                _marker: PhantomData,
            },
            closure,
        });
        Self {
            inner: NonNull::new(Box::into_raw(inner)).expect("Box::into_raw is non-null"),
        }
    }

    /// Create a new heap-allocated block from a zero-argument closure.
    pub fn new0(closure: F) -> Self
    where
        F: Fn() + 'static,
    {
        extern "C" fn invoke_impl<F: Fn()>(block: *const RcBlockInner<F>) {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure();
        }
        Self::make(closure, invoke_impl::<F> as *const c_void)
    }

    /// Create a new heap-allocated block from a single-argument closure.
    pub fn new<A: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A) + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A), A: Copy>(block: *const RcBlockInner<F>, a: A) {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a);
        }
        Self::make(closure, invoke_impl::<F, A> as *const c_void)
    }

    /// Create a new heap-allocated block from a single-argument closure returning `R`.
    pub fn new_ret<A: 'static + Copy, R: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A) -> R + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A) -> R, A: Copy, R: Copy>(
            block: *const RcBlockInner<F>,
            a: A,
        ) -> R {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a)
        }
        Self::make(closure, invoke_impl::<F, A, R> as *const c_void)
    }

    /// Create a new heap-allocated block from a two-argument closure.
    pub fn new2<A: 'static + Copy, B: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A, B) + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A, B), A: Copy, B: Copy>(
            block: *const RcBlockInner<F>,
            a: A,
            b: B,
        ) {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a, b);
        }
        Self::make(closure, invoke_impl::<F, A, B> as *const c_void)
    }

    /// Create a new heap-allocated block from a three-argument closure.
    pub fn new3<A: 'static + Copy, B: 'static + Copy, C: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A, B, C) + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A, B, C), A: Copy, B: Copy, C: Copy>(
            block: *const RcBlockInner<F>,
            a: A,
            b: B,
            c: C,
        ) {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a, b, c);
        }
        Self::make(closure, invoke_impl::<F, A, B, C> as *const c_void)
    }

    /// Create a new heap-allocated block from a four-argument closure.
    pub fn new4<A: 'static + Copy, B: 'static + Copy, C: 'static + Copy, D: 'static + Copy>(
        closure: F,
    ) -> Self
    where
        F: Fn(A, B, C, D) + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A, B, C, D), A: Copy, B: Copy, C: Copy, D: Copy>(
            block: *const RcBlockInner<F>,
            a: A,
            b: B,
            c: C,
            d: D,
        ) {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // allocated by `Box::into_raw`; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a, b, c, d);
        }
        Self::make(closure, invoke_impl::<F, A, B, C, D> as *const c_void)
    }
}

impl<F> RcBlock<F> {
    /// Call this block directly from Rust and return its value.
    pub fn call_ret<A: Copy, R>(&self, a: A) -> R
    where
        F: Fn(A) -> R,
    {
        // SAFETY: `self.inner` is a valid non-null pointer to a live `RcBlockInner<F>`;
        // we hold `&self` so it cannot be dropped during this call.
        let closure = unsafe { &self.inner.as_ref().closure };
        closure(a)
    }
}

impl<F> Deref for RcBlock<F> {
    type Target = Block<F>;
    fn deref(&self) -> &Block<F> {
        // SAFETY: `self.inner` was produced by `NonNull::new(Box::into_raw(...))` in `make`
        // and is valid until `Drop::drop` runs; we hold `&self` so it cannot be dropped here.
        unsafe { &self.inner.as_ref().block }
    }
}

impl<F> Drop for RcBlock<F> {
    fn drop(&mut self) {
        // SAFETY: `self.inner` was produced by `Box::into_raw` in `make`; reconstructing
        // the `Box` here is the exactly paired deallocation and runs the closure destructor.
        unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    use super::*;

    fn as_dyn<A: 'static + Copy, F: Fn(A) + 'static>(block: &RcBlock<F>) -> &Block<dyn Fn(A)> {
        // SAFETY: `Block<F>` and `Block<dyn Fn(A)>` have identical `repr(C)` layouts;
        // `PhantomData<*const F>` is zero-sized so the bit-pattern is the same either way.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A)>) }
    }

    fn as_dyn0<F: Fn() + 'static>(block: &RcBlock<F>) -> &Block<dyn Fn()> {
        // SAFETY: identical repr(C) layouts; PhantomData<*const F> is zero-sized.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn()>) }
    }

    fn as_dyn2<A: 'static + Copy, B: 'static + Copy, F: Fn(A, B) + 'static>(
        block: &RcBlock<F>,
    ) -> &Block<dyn Fn(A, B)> {
        // SAFETY: identical repr(C) layouts; PhantomData<*const F> is zero-sized.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A, B)>) }
    }

    fn as_dyn3<
        A: 'static + Copy,
        B: 'static + Copy,
        C: 'static + Copy,
        F: Fn(A, B, C) + 'static,
    >(
        block: &RcBlock<F>,
    ) -> &Block<dyn Fn(A, B, C)> {
        // SAFETY: identical repr(C) layouts; PhantomData<*const F> is zero-sized.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A, B, C)>) }
    }

    fn as_dyn4<
        A: 'static + Copy,
        B: 'static + Copy,
        C: 'static + Copy,
        D: 'static + Copy,
        F: Fn(A, B, C, D) + 'static,
    >(
        block: &RcBlock<F>,
    ) -> &Block<dyn Fn(A, B, C, D)> {
        // SAFETY: identical repr(C) layouts; PhantomData<*const F> is zero-sized.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A, B, C, D)>) }
    }

    #[test]
    fn test_block_call_0_args() {
        static CALLED: AtomicBool = AtomicBool::new(false);
        let block = RcBlock::new0(|| {
            CALLED.store(true, Ordering::SeqCst);
        });
        as_dyn0(&block).call(());
        assert!(CALLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_block_call_1_arg() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new::<i32>(|x: i32| {
            RESULT.store(x * 2, Ordering::SeqCst);
        });
        as_dyn(&block).call((21,));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_call_via_ref() {
        // Simulate how bwebview passes &*block to ObjC and then Rust calls it
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new::<i32>(|x: i32| {
            RESULT.store(x + 10, Ordering::SeqCst);
        });
        let block_ref: &Block<dyn Fn(i32)> = as_dyn(&block);
        block_ref.call((32,));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_capture() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let multiplier = 7i32;
        let block = RcBlock::new::<i32>(move |x: i32| {
            RESULT.store(x * multiplier, Ordering::SeqCst);
        });
        as_dyn(&block).call((6,));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_call_2_args() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new2::<i32, i32>(|a: i32, b: i32| {
            RESULT.store((a * 10) + b, Ordering::SeqCst);
        });
        as_dyn2(&block).call((4, 2));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_call_3_args() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new3::<i32, i32, i32>(|a: i32, b: i32, c: i32| {
            RESULT.store((a * b) + c, Ordering::SeqCst);
        });
        as_dyn3(&block).call((8, 5, 2));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_call_4_args() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new4::<i32, i32, i32, i32>(|a: i32, b: i32, c: i32, d: i32| {
            RESULT.store(a + b + c + d, Ordering::SeqCst);
        });
        as_dyn4(&block).call((10, 11, 12, 9));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_drop_runs_closure_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        struct DropGuard(Arc<AtomicBool>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let guard = DropGuard(dropped.clone());
        let block = RcBlock::new::<i32>(move |_: i32| {
            let _ = &guard;
        });
        assert!(!dropped.load(Ordering::SeqCst));
        drop(block);
        assert!(
            dropped.load(Ordering::SeqCst),
            "closure should be dropped with RcBlock"
        );
    }

    #[test]
    fn test_block_call_1_arg_ret() {
        let block = RcBlock::new_ret::<i32, i32>(|x: i32| x * 2);
        assert_eq!(block.call_ret(21), 42);
    }

    #[test]
    fn test_block_encode() {
        assert_eq!(<&Block<dyn Fn(i32)>>::ENCODING.to_string(), "@?");
    }
}
