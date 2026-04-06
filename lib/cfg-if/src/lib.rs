/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [cfg-if](https://crates.io/crates/cfg-if) crate

/// Evaluate a series of `#[cfg]` conditions and emit the code for the first matching branch.
///
/// # Example
/// ```rust
/// cfg_if::cfg_if! {
///     if #[cfg(unix)] {
///         fn platform() -> &'static str { "unix" }
///     } else if #[cfg(windows)] {
///         fn platform() -> &'static str { "windows" }
///     } else {
///         fn platform() -> &'static str { "other" }
///     }
/// }
/// ```
#[macro_export]
macro_rules! cfg_if {
    // if/else-if chain with a final else
    ($(
        if #[cfg($i_met:meta)] { $($i_tokens:tt)* }
    ) else + else {
        $($e_tokens:tt)*
    }) => {
        $crate::cfg_if! {
            @inner () ;
            $( ( ($i_met) ($($i_tokens)*) ), )+
            ( () ($($e_tokens)*) ),
        }
    };

    // if/else-if chain without a final else
    ($(
        if #[cfg($i_met:meta)] { $($i_tokens:tt)* }
    ) else *) => {
        $crate::cfg_if! {
            @inner () ;
            $( ( ($i_met) ($($i_tokens)*) ), )*
        }
    };

    // Inner accumulator - base case: nothing left to process
    (@inner ($($not:meta,)*) ;) => {};

    // Inner accumulator - emit one branch and recurse
    (@inner ($($not:meta,)*) ; ( ($($m:meta),*) ($($tt:tt)*) ), $($rest:tt)*) => {
        // Route through @emit so #[cfg] applies to a macro invocation, which is
        // valid at both item level (module scope) and statement level (inside fns).
        #[cfg(all($($m,)* not(any($($not),*))))]
        $crate::cfg_if! { @emit $($tt)* }
        $crate::cfg_if! {
            @inner ($($not,)* $($m,)*) ;
            $($rest)*
        }
    };

    // Re-emit tokens unchanged (items or statements depending on call site)
    (@emit $($tt:tt)*) => { $($tt)* };
}
