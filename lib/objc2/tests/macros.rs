/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Integration tests for `define_class!` and `extern_class!`.
//!
//! These live in an integration test file rather than inline `#[cfg(test)]` because both macros
//! generate `::objc2::` absolute paths in their output, which only resolve when the caller is an
//! external crate -- as integration tests are.

#![cfg(target_vendor = "apple")]
#![allow(unsafe_code, clippy::undocumented_unsafe_blocks)]

use std::cell::Cell;

use objc2::runtime::AnyObject;
use objc2::{class, define_class, extern_class, msg_send};

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

// MARK: define_class! - no methods

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestNoMethodsClass"]
    struct NoMethodsClass;
);

#[test]
fn test_define_class_no_methods_class_ptr_non_null() {
    assert!(!NoMethodsClass::class().is_null());
}

#[test]
fn test_define_class_class_idempotent() {
    assert_eq!(NoMethodsClass::class(), NoMethodsClass::class());
}

#[test]
fn test_define_class_instantiation() {
    let obj: *mut AnyObject = unsafe { msg_send![NoMethodsClass::class(), new] };
    assert!(!obj.is_null());
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

// MARK: define_class! - with methods

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestMethodClass"]
    struct MethodClass;

    impl MethodClass {
        #[unsafe(method(answer))]
        const fn _answer(&self) -> i64 {
            42
        }

        #[unsafe(method(double:))]
        const fn _double(&self, n: i64) -> i64 {
            n * 2
        }

        #[unsafe(method(add:to:))]
        const fn _add(&self, a: i64, b: i64) -> i64 {
            a + b
        }
    }
);

#[test]
fn test_define_class_zero_arg_method_return() {
    let obj: *mut AnyObject = unsafe { msg_send![MethodClass::class(), new] };
    let result: i64 = unsafe { msg_send![obj, answer] };
    assert_eq!(result, 42);
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

#[test]
fn test_define_class_one_arg_method_return() {
    let obj: *mut AnyObject = unsafe { msg_send![MethodClass::class(), new] };
    let result: i64 = unsafe { msg_send![obj, double: 7i64] };
    assert_eq!(result, 14);
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

#[test]
fn test_define_class_two_arg_method_return() {
    let obj: *mut AnyObject = unsafe { msg_send![MethodClass::class(), new] };
    let result: i64 = unsafe { msg_send![obj, add: 3i64, to: 4i64] };
    assert_eq!(result, 7);
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

// MARK: define_class! - with ivars

struct CounterIvars {
    count: Cell<i64>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestCounterClass"]
    #[ivars = CounterIvars]
    struct CounterClass;

    impl CounterClass {
        #[unsafe(method(increment))]
        fn _increment(&self) {
            let c = self.ivars().count.get();
            self.ivars().count.set(c + 1);
        }

        #[unsafe(method(count))]
        fn _count(&self) -> i64 {
            self.ivars().count.get()
        }
    }
);

#[test]
fn test_define_class_ivars_zero_initialized() {
    let obj: *mut AnyObject = unsafe { msg_send![CounterClass::class(), alloc] };
    let obj: *mut AnyObject = unsafe { msg_send![obj, init] };
    let count: i64 = unsafe { msg_send![obj, count] };
    assert_eq!(count, 0);
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

#[test]
fn test_define_class_ivars_mutated_via_method() {
    let obj: *mut AnyObject = unsafe { msg_send![CounterClass::class(), alloc] };
    let obj: *mut AnyObject = unsafe { msg_send![obj, init] };
    unsafe {
        let _: () = msg_send![obj, increment];
    }
    unsafe {
        let _: () = msg_send![obj, increment];
    }
    unsafe {
        let _: () = msg_send![obj, increment];
    }
    let count: i64 = unsafe { msg_send![obj, count] };
    assert_eq!(count, 3);
    unsafe {
        let _: () = msg_send![obj, release];
    }
}

#[test]
fn test_define_class_ivars_independent_per_instance() {
    let a: *mut AnyObject = unsafe { msg_send![CounterClass::class(), alloc] };
    let a: *mut AnyObject = unsafe { msg_send![a, init] };
    let b: *mut AnyObject = unsafe { msg_send![CounterClass::class(), alloc] };
    let b: *mut AnyObject = unsafe { msg_send![b, init] };

    unsafe {
        let _: () = msg_send![a, increment];
    }
    unsafe {
        let _: () = msg_send![a, increment];
    }
    unsafe {
        let _: () = msg_send![b, increment];
    }

    let count_a: i64 = unsafe { msg_send![a, count] };
    let count_b: i64 = unsafe { msg_send![b, count] };
    assert_eq!(count_a, 2);
    assert_eq!(count_b, 1);

    unsafe {
        let _: () = msg_send![a, release];
    }
    unsafe {
        let _: () = msg_send![b, release];
    }
}

// MARK: extern_class!

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSObject"]
    struct ExternNSObject;
);

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSString"]
    struct ExternNSString;
);

#[test]
fn test_extern_class_returns_non_null() {
    assert!(!ExternNSObject::class().is_null());
}

#[test]
fn test_extern_class_matches_class_macro() {
    assert_eq!(ExternNSObject::class(), class!(NSObject));
}

#[test]
fn test_extern_class_name_attr_selects_objc_class() {
    assert_eq!(ExternNSString::class(), class!(NSString));
}
