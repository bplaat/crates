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
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

use objc2::encode::{Encode, EncodeArgument, EncodeArguments, EncodeReturn, Encoding};

#[link(name = "System", kind = "dylib")]
unsafe extern "C-unwind" {
    static _NSConcreteStackBlock: u8;
    fn _Block_copy(block: *const c_void) -> *mut c_void;
    fn _Block_release(block: *const c_void);
}

#[repr(C)]
struct BlockDescriptor {
    reserved: usize,
    size: usize,
    copy: unsafe extern "C-unwind" fn(*mut c_void, *const c_void),
    dispose: unsafe extern "C-unwind" fn(*mut c_void),
}

const BLOCK_HAS_COPY_DISPOSE: i32 = 1 << 25;

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

// A block argument is encoded as `@?` (an object which is a block)
// SAFETY: A block pointer always has encoding `@?` in the ObjC type system.
unsafe impl<F: ?Sized> Encode for &Block<F> {
    const ENCODING: Encoding = Encoding::Block;
}

macro_rules! impl_block_call {
    ($($t:ident: $a:ident),*) => {
        impl<R: 'static + Copy + EncodeReturn, $($t: 'static + Copy + EncodeArgument),*>
            Block<dyn Fn($($t),*) -> R>
        {
            /// Call this block with the given arguments.
            pub fn call(&self, ($($a,)*): ($($t,)*)) -> R {
                // SAFETY: `_invoke` was stored by `RcBlock::make` with exactly this signature.
                // The block ABI stores all invoke pointers as opaque
                // `*const c_void`, so the transmute is required. `self` is a valid shared
                // reference that remains valid for the duration of this call.
                unsafe {
                    let invoke: unsafe extern "C-unwind" fn(*const c_void $(, $t)*) -> R =
                        std::mem::transmute(self._invoke);
                    invoke(self as *const Self as *const c_void $(, $a)*)
                }
            }
        }
    };
}
impl_block_call!(A: a);
impl_block_call!(A: a, B: b);

// Inner heap layout for `RcBlock`: ObjC block header immediately followed by the closure.
#[repr(C)]
struct RcBlockInner<F: ?Sized, C> {
    block: Block<F>,
    closure: C,
}

struct BlockDescriptorFor<F: ?Sized, C>(PhantomData<(*const F, C)>);

impl<F: ?Sized, C> BlockDescriptorFor<F, C> {
    const VALUE: BlockDescriptor = BlockDescriptor {
        reserved: 0,
        size: size_of::<RcBlockInner<F, C>>(),
        copy: copy_closure::<C>,
        dispose: dispose_closure::<F, C>,
    };
}

const unsafe extern "C-unwind" fn copy_closure<F>(_: *mut c_void, _: *const c_void) {
    // The initial stack block is copied exactly once. The runtime has already moved the closure
    // bytes into the heap block, and RcBlock::make forgets the stack copy.
}

unsafe extern "C-unwind" fn dispose_closure<F: ?Sized, C>(block: *mut c_void) {
    // SAFETY: The Blocks runtime calls this exactly once when the heap block's final reference is
    // released. RcBlock::make transferred ownership of the closure into that block.
    unsafe {
        std::ptr::drop_in_place(std::ptr::addr_of_mut!(
            (*block.cast::<RcBlockInner<F, C>>()).closure
        ));
    }
}

/// A heap-allocated ObjC block wrapping a Rust closure.
pub struct RcBlock<F: ?Sized> {
    inner: NonNull<Block<F>>,
}

impl<F: ?Sized> RcBlock<F> {
    fn make<C>(closure: C, invoke: *const c_void) -> Self {
        let stack = RcBlockInner {
            block: Block {
                // SAFETY: `_NSConcreteStackBlock` is a valid extern static exported by
                // libSystem; it is always initialised before any Rust code runs.
                _isa: std::ptr::addr_of!(_NSConcreteStackBlock).cast(),
                _flags: BLOCK_HAS_COPY_DISPOSE,
                _reserved: 0,
                _invoke: invoke,
                _descriptor: &BlockDescriptorFor::<F, C>::VALUE,
                _marker: PhantomData,
            },
            closure,
        };
        let mut stack = ManuallyDrop::new(stack);
        // SAFETY: `stack` has the Objective-C stack block layout. `_Block_copy` promotes it to
        // independently owned heap storage. The no-op copy helper transfers the closure bytes.
        let inner = unsafe { _Block_copy((&stack.block as *const Block<F>).cast()) };
        let Some(inner) = NonNull::new(inner.cast()) else {
            // SAFETY: No heap block was created, so ownership still belongs to the stack value.
            unsafe { ManuallyDrop::drop(&mut stack) };
            panic!("_Block_copy returned null");
        };
        Self { inner }
    }

    /// Create a new heap-allocated block from a closure.
    pub fn new<'f, A, R, Closure>(closure: Closure) -> Self
    where
        A: EncodeArguments,
        R: EncodeReturn,
        Closure: IntoBlock<'f, A, R, Dyn = F>,
    {
        closure.into_block()
    }
}

impl<F> RcBlock<F> {
    /// Create a new heap-allocated block from a single-argument closure returning `R`.
    pub fn new_ret<A: 'static + Copy + EncodeArgument, R: 'static + Copy + EncodeReturn>(
        closure: F,
    ) -> Self
    where
        F: Fn(A) -> R + 'static,
    {
        extern "C-unwind" fn invoke_impl<F: Fn(A) -> R, A: Copy, R: Copy>(
            block: *const RcBlockInner<F, F>,
            a: A,
        ) -> R {
            // SAFETY: `block` is a valid non-null pointer to a live `RcBlockInner<F>`
            // owned by the invoking block; it stays alive for the duration of this call.
            let closure = unsafe { &(*block).closure };
            closure(a)
        }
        Self::make(closure, invoke_impl::<F, A, R> as *const c_void)
    }
}

/// Conversion from a Rust closure to its dynamic Objective-C block signature.
///
/// # Safety
///
/// Implementations must use a dynamic function signature matching `A` and `R`.
pub unsafe trait IntoBlock<'f, A, R>
where
    A: EncodeArguments,
    R: EncodeReturn,
{
    /// The dynamic function type stored in the block.
    type Dyn: ?Sized;

    #[doc(hidden)]
    fn into_block(self) -> RcBlock<Self::Dyn>;
}

// SAFETY: The invoke function uses the same one-argument ABI as the dynamic signature.
unsafe impl<'f, T, R, C> IntoBlock<'f, (T,), R> for C
where
    T: 'static + Copy + EncodeArgument,
    R: 'static + Copy + EncodeReturn,
    C: Fn(T) -> R + 'f,
{
    type Dyn = dyn Fn(T) -> R + 'f;

    fn into_block(self) -> RcBlock<Self::Dyn> {
        extern "C-unwind" fn invoke<C: Fn(T) -> R, T: Copy, R: Copy>(
            block: *const RcBlockInner<dyn Fn(T) -> R, C>,
            value: T,
        ) -> R {
            // SAFETY: The block stores `C` directly after a header with this dynamic signature.
            unsafe { ((*block).closure)(value) }
        }
        RcBlock::make(self, invoke::<C, T, R> as *const c_void)
    }
}

// SAFETY: The invoke function uses the same two-argument ABI as the dynamic signature.
unsafe impl<'f, T, U, R, C> IntoBlock<'f, (T, U), R> for C
where
    T: 'static + Copy + EncodeArgument,
    U: 'static + Copy + EncodeArgument,
    R: 'static + Copy + EncodeReturn,
    C: Fn(T, U) -> R + 'f,
{
    type Dyn = dyn Fn(T, U) -> R + 'f;

    fn into_block(self) -> RcBlock<Self::Dyn> {
        extern "C-unwind" fn invoke<C: Fn(T, U) -> R, T: Copy, U: Copy, R: Copy>(
            block: *const RcBlockInner<dyn Fn(T, U) -> R, C>,
            first: T,
            second: U,
        ) -> R {
            // SAFETY: The block stores `C` directly after a header with this dynamic signature.
            unsafe { ((*block).closure)(first, second) }
        }
        RcBlock::make(self, invoke::<C, T, U, R> as *const c_void)
    }
}

impl<F: ?Sized> Block<F> {
    /// Copy this block to the heap or retain it if it is already heap allocated.
    pub fn copy(&self) -> RcBlock<F> {
        // SAFETY: `self` is a live Objective-C block. `_Block_copy` returns an owned heap block
        // with the same function signature.
        let inner = unsafe { _Block_copy((self as *const Self).cast()) };
        RcBlock {
            inner: NonNull::new(inner.cast()).expect("_Block_copy returned null"),
        }
    }
}

impl<F: ?Sized> Clone for RcBlock<F> {
    fn clone(&self) -> Self {
        self.deref().copy()
    }
}

impl<F: ?Sized> Deref for RcBlock<F> {
    type Target = Block<F>;
    fn deref(&self) -> &Block<F> {
        // SAFETY: `self.inner` is the live block returned by `_Block_copy` in `make` and remains
        // valid until `Drop::drop`; we hold `&self` so it cannot be dropped here.
        unsafe { self.inner.as_ref() }
    }
}

impl<F: ?Sized> Drop for RcBlock<F> {
    fn drop(&mut self) {
        // SAFETY: `self.inner` owns the reference returned by `_Block_copy` in `make`.
        unsafe { _Block_release(self.inner.as_ptr().cast()) };
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    use super::*;

    fn as_dyn_ret<A: 'static + Copy, R: 'static + Copy, F: Fn(A) -> R + 'static>(
        block: &RcBlock<F>,
    ) -> &Block<dyn Fn(A) -> R> {
        // SAFETY: identical repr(C) layouts; PhantomData<*const F> is zero-sized.
        unsafe { &*((&**block) as *const Block<F> as *const Block<dyn Fn(A) -> R>) }
    }

    #[test]
    fn test_block_call_1_arg() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new::<(i32,), (), _>(|x: i32| {
            RESULT.store(x * 2, Ordering::SeqCst);
        });
        block.call((21,));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_call_via_ref() {
        // Simulate how bwebview passes &*block to ObjC and then Rust calls it
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let block = RcBlock::new::<(i32,), (), _>(|x: i32| {
            RESULT.store(x + 10, Ordering::SeqCst);
        });
        let block_ref: &Block<dyn Fn(i32)> = &block;
        block_ref.call((32,));
        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_block_capture() {
        static RESULT: AtomicI32 = AtomicI32::new(0);
        let multiplier = 7i32;
        let block = RcBlock::new::<(i32,), (), _>(move |x: i32| {
            RESULT.store(x * multiplier, Ordering::SeqCst);
        });
        block.call((6,));
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
        let block = RcBlock::new::<(i32,), (), _>(move |_: i32| {
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
    fn test_system_block_copy_keeps_capture_alive() {
        let dropped = Arc::new(AtomicBool::new(false));
        struct DropGuard(Arc<AtomicBool>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let guard = DropGuard(dropped.clone());
        let block = RcBlock::new::<(i32,), (), _>(move |_: i32| {
            let _ = &guard;
        });
        // SAFETY: `block` is a valid Objective-C block and remains alive for this call.
        let copied = unsafe { _Block_copy((&*block as *const Block<_>).cast()) };
        drop(block);
        assert!(!dropped.load(Ordering::SeqCst));

        // SAFETY: `_Block_copy` returned a retained copy of the same block signature.
        let copied = unsafe { &*copied.cast::<Block<dyn Fn(i32)>>() };
        copied.call((0,));
        // SAFETY: This balances the single ownership reference returned by `_Block_copy`.
        unsafe { _Block_release((copied as *const Block<dyn Fn(i32)>).cast()) };
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn test_clone_keeps_capture_alive() {
        let dropped = Arc::new(AtomicBool::new(false));
        struct DropGuard(Arc<AtomicBool>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let guard = DropGuard(dropped.clone());
        let block = RcBlock::new::<(i32,), (), _>(move |_| {
            let _ = &guard;
        });
        let cloned = block.clone();
        drop(block);
        assert!(!dropped.load(Ordering::SeqCst));
        cloned.call((0,));
        drop(cloned);
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn test_block_is_objective_c_object() {
        let block = RcBlock::new::<(i32,), (), _>(|_| {});
        // SAFETY: A block is an Objective-C object and responds to the NSObject self selector.
        unsafe {
            let object = (&*block as *const Block<_>).cast::<AnyObject>().cast_mut();
            let same: *mut AnyObject = msg_send![object, self];
            assert_eq!(same, object);
        }
    }

    #[test]
    fn test_block_call_1_arg_ret() {
        let block = RcBlock::new_ret::<i32, i32>(|x: i32| x * 2);
        assert_eq!(as_dyn_ret(&block).call((21,)), 42);
    }

    #[test]
    fn test_block_encode() {
        assert_eq!(<&Block<dyn Fn(i32)>>::ENCODING.to_string(), "@?");
    }
}
