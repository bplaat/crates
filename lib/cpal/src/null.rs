/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Backend for unsupported platforms, it has no devices.

use crate::{DataCallback, Error, ErrorCallback, SampleRate, StreamConfig};

pub(crate) enum Device {}

pub(crate) const fn default_input_device() -> Option<Device> {
    None
}

impl Device {
    pub(crate) const fn default_sample_rate(&self) -> Result<SampleRate, Error> {
        match *self {}
    }

    pub(crate) fn build_input_stream(
        &self,
        _config: &StreamConfig,
        _data: DataCallback,
        _error: ErrorCallback,
    ) -> Result<Stream, Error> {
        match *self {}
    }
}

pub(crate) enum Stream {}

impl Stream {
    pub(crate) const fn play(&self) -> Result<(), Error> {
        match *self {}
    }

    pub(crate) const fn pause(&self) -> Result<(), Error> {
        match *self {}
    }
}
