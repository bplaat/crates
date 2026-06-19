/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::ffi::{objc_autoreleasePoolPop, objc_autoreleasePoolPush};

/// A token representing an active autorelease pool
pub struct AutoreleasePool(());

/// Run a closure within an autorelease pool
pub fn autoreleasepool<F, R>(f: F) -> R
where
    F: FnOnce(&AutoreleasePool) -> R,
{
    // SAFETY: `objc_autoreleasePoolPush` and `objc_autoreleasePoolPop` must be called in
    // matched pairs on the same thread. The token from Push is immediately passed back to
    // Pop after the closure returns, satisfying the ObjC runtime's stack discipline.
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let result = f(&AutoreleasePool(()));
        objc_autoreleasePoolPop(pool);
        result
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_autoreleasepool_runs_closure() {
        let mut ran = false;
        autoreleasepool(|_| {
            ran = true;
        });
        assert!(ran);
    }

    #[test]
    fn test_autoreleasepool_returns_value() {
        let result = autoreleasepool(|_| 42_i32);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_autoreleasepool_nested() {
        let result = autoreleasepool(|_| autoreleasepool(|_| 7_i32));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_autoreleasepool_token_accessible() {
        autoreleasepool(|pool| {
            let _: &AutoreleasePool = pool;
        });
    }

    #[test]
    fn test_autoreleasepool_with_objc_objects() {
        #[link(name = "Foundation", kind = "framework")]
        unsafe extern "C" {}

        autoreleasepool(|_| {
            // SAFETY: NSObject is a valid Foundation class; new returns a fully initialized object;
            // autorelease hands ownership to the pool, which releases it when drained.
            unsafe {
                let obj: *mut crate::runtime::AnyObject =
                    crate::msg_send![crate::class!(NSObject), new];
                assert!(!obj.is_null());
                let _: *mut crate::runtime::AnyObject = crate::msg_send![obj, autorelease];
            }
        });
    }
}
