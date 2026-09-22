/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [digest](https://crates.io/crates/digest) crate

pub use crypto_common::{BlockSizeUser, Output, OutputSizeUser, Reset};

/// Marker trait for cryptographic hash functions.
pub trait HashMarker {}

/// Incremental input for a cryptographic primitive.
pub trait Update {
    /// Feeds bytes into this value.
    fn update(&mut self, data: &[u8]);

    /// Feeds bytes and returns this value.
    fn chain(mut self, data: impl AsRef<[u8]>) -> Self
    where
        Self: Sized,
    {
        self.update(data.as_ref());
        self
    }
}

/// A cryptographic primitive with a fixed-size output.
pub trait FixedOutput: Update + OutputSizeUser + Sized {
    /// Finalizes into the provided output buffer.
    fn finalize_into(self, out: &mut Output<Self>);

    /// Finalizes and returns the output.
    fn finalize_fixed(self) -> Output<Self> {
        let mut output = Output::<Self>::default();
        self.finalize_into(&mut output);
        output
    }
}

/// A fixed-output primitive that can reset itself while finalizing.
pub trait FixedOutputReset: FixedOutput + Reset {
    /// Finalizes into the provided buffer and resets the value.
    fn finalize_into_reset(&mut self, out: &mut Output<Self>);

    /// Finalizes, returns the output, and resets the value.
    fn finalize_fixed_reset(&mut self) -> Output<Self> {
        let mut output = Output::<Self>::default();
        self.finalize_into_reset(&mut output);
        output
    }
}

// MARK: Digest
/// Convenience trait for fixed-output hash functions.
pub trait Digest: OutputSizeUser {
    /// Creates a new hasher.
    fn new() -> Self;
    /// Creates a new hasher and feeds it a prefix.
    fn new_with_prefix(data: impl AsRef<[u8]>) -> Self;
    /// Feeds bytes into this hasher.
    fn update(&mut self, data: impl AsRef<[u8]>);
    /// Feeds bytes and returns this hasher.
    fn chain_update(self, data: impl AsRef<[u8]>) -> Self;
    /// Finalizes and returns the output.
    fn finalize(self) -> Output<Self>;
    /// Finalizes into the provided output buffer.
    fn finalize_into(self, out: &mut Output<Self>);
    /// Finalizes, returns the output, and resets the hasher.
    fn finalize_reset(&mut self) -> Output<Self>
    where
        Self: FixedOutputReset;
    /// Finalizes into a buffer and resets the hasher.
    fn finalize_into_reset(&mut self, out: &mut Output<Self>)
    where
        Self: FixedOutputReset;
    /// Computes a digest in one operation.
    fn digest(data: impl AsRef<[u8]>) -> Output<Self>;
    /// Returns the output size in bytes.
    fn output_size() -> usize;
    /// Resets the hasher.
    fn reset(&mut self)
    where
        Self: Reset;
}

impl<D> Digest for D
where
    D: FixedOutput + Default + Update + HashMarker,
{
    fn new() -> Self {
        Self::default()
    }

    fn new_with_prefix(data: impl AsRef<[u8]>) -> Self {
        let mut value = Self::default();
        Update::update(&mut value, data.as_ref());
        value
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Update::update(self, data.as_ref());
    }

    fn chain_update(mut self, data: impl AsRef<[u8]>) -> Self {
        Update::update(&mut self, data.as_ref());
        self
    }

    fn finalize(self) -> Output<Self> {
        self.finalize_fixed()
    }

    fn finalize_into(self, out: &mut Output<Self>) {
        FixedOutput::finalize_into(self, out);
    }

    fn finalize_reset(&mut self) -> Output<Self>
    where
        Self: FixedOutputReset,
    {
        self.finalize_fixed_reset()
    }

    fn finalize_into_reset(&mut self, out: &mut Output<Self>)
    where
        Self: FixedOutputReset,
    {
        FixedOutputReset::finalize_into_reset(self, out);
    }

    fn digest(data: impl AsRef<[u8]>) -> Output<Self> {
        Self::new_with_prefix(data).finalize_fixed()
    }

    fn output_size() -> usize {
        <Self as OutputSizeUser>::output_size()
    }

    fn reset(&mut self)
    where
        Self: Reset,
    {
        Reset::reset(self);
    }
}
