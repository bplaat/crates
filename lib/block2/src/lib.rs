/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [block2](https://crates.io/crates/block2) crate

#![cfg(target_vendor = "apple")]

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

const BLOCK_NEEDS_FREE: i32 = 1 << 24;

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

unsafe impl<F: ?Sized> Send for Block<F> {}
unsafe impl<F: ?Sized> Sync for Block<F> {}

// A block argument is encoded as `@?` (an object which is a block)
unsafe impl<F: ?Sized> Encode for &Block<F> {
    const ENCODING: Encoding = Encoding::Block;
}

macro_rules! impl_block_call {
    ($($t:ident: $a:ident),*) => {
        impl<$($t: 'static + Copy),*> Block<dyn Fn($($t,)*)> {
            /// Call this block with the given arguments.
            pub fn call(&self, ($($a,)*): ($($t,)*)) {
                let invoke: unsafe extern "C" fn(*const c_void $(, $t)*) =
                    unsafe { std::mem::transmute(self._invoke) };
                unsafe { invoke(self as *const Self as *const c_void $(, $a)*) };
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

unsafe impl<F: Send> Send for RcBlock<F> {}
unsafe impl<F: Sync> Sync for RcBlock<F> {}

impl<F> RcBlock<F> {
    fn make(closure: F, invoke: *const c_void) -> Self {
        let inner = Box::new(RcBlockInner {
            block: Block {
                _isa: unsafe { _NSConcreteMallocBlock },
                _flags: BLOCK_NEEDS_FREE,
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
            unsafe { ((*block).closure)() };
        }
        Self::make(closure, invoke_impl::<F> as *const c_void)
    }

    /// Create a new heap-allocated block from a single-argument closure.
    pub fn new<A: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A) + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A), A: Copy>(block: *const RcBlockInner<F>, a: A) {
            unsafe { ((*block).closure)(a) };
        }
        Self::make(closure, invoke_impl::<F, A> as *const c_void)
    }

    /// Create a new heap-allocated block from a single-argument closure that returns a value.
    pub fn new_ret<A: 'static + Copy, R: 'static + Copy>(closure: F) -> Self
    where
        F: Fn(A) -> R + 'static,
    {
        extern "C" fn invoke_impl<F: Fn(A) -> R, A: Copy, R: Copy>(
            block: *const RcBlockInner<F>,
            a: A,
        ) -> R {
            unsafe { ((*block).closure)(a) }
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
            unsafe { ((*block).closure)(a, b) };
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
            unsafe { ((*block).closure)(a, b, c) };
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
            unsafe { ((*block).closure)(a, b, c, d) };
        }
        Self::make(closure, invoke_impl::<F, A, B, C, D> as *const c_void)
    }
}

impl<F> Deref for RcBlock<F> {
    type Target = Block<F>;
    fn deref(&self) -> &Block<F> {
        unsafe { &self.inner.as_ref().block }
    }
}

impl<F> Drop for RcBlock<F> {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    use super::*;

    // Cast a concrete RcBlock reference to a dyn-typed Block reference for calling from Rust.
    // Safe because Block<F> and Block<dyn Fn(A)> have identical layouts (PhantomData is 0 bytes).
    fn as_dyn<A: 'static + Copy, F: Fn(A) + 'static>(block: &RcBlock<F>) -> &Block<dyn Fn(A)> {
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A)>) }
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
    fn test_block_encode() {
        assert_eq!(<&Block<dyn Fn(i32)>>::ENCODING.to_string(), "@?");
    }
}
