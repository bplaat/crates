/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! ALSA backend, the `default` device converts the format, rate and channels.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_int, c_uint, c_ulong, c_void};
use std::ptr::{self, NonNull};

use crate::capture::{self, Capture, DEFAULT_SAMPLE_RATE};
use crate::{DataCallback, Error, ErrorCallback, ErrorKind, SampleRate, StreamConfig};

mod sys {
    use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

    #[repr(C)]
    pub(super) struct SndPcm {
        _private: [u8; 0],
    }

    pub(super) const SND_PCM_STREAM_CAPTURE: c_int = 1;
    pub(super) const SND_PCM_NONBLOCK: c_int = 1;
    pub(super) const SND_PCM_FORMAT_FLOAT_LE: c_int = 14;
    pub(super) const SND_PCM_ACCESS_RW_INTERLEAVED: c_int = 3;

    #[link(name = "libasound.so.2", kind = "dylib", modifiers = "+verbatim")]
    unsafe extern "C" {
        pub(super) fn snd_pcm_open(
            pcm: *mut *mut SndPcm,
            name: *const c_char,
            stream: c_int,
            mode: c_int,
        ) -> c_int;
        pub(super) fn snd_pcm_close(pcm: *mut SndPcm) -> c_int;
        pub(super) fn snd_pcm_set_params(
            pcm: *mut SndPcm,
            format: c_int,
            access: c_int,
            channels: c_uint,
            rate: c_uint,
            soft_resample: c_int,
            latency: c_uint,
        ) -> c_int;
        pub(super) fn snd_pcm_prepare(pcm: *mut SndPcm) -> c_int;
        pub(super) fn snd_pcm_drop(pcm: *mut SndPcm) -> c_int;
        pub(super) fn snd_pcm_readi(pcm: *mut SndPcm, buffer: *mut c_void, size: c_ulong)
        -> c_long;
        pub(super) fn snd_pcm_recover(pcm: *mut SndPcm, err: c_int, silent: c_int) -> c_int;
        pub(super) fn snd_strerror(errnum: c_int) -> *const c_char;
    }
}

const DEVICE_NAME: &CStr = c"default";

const ENOENT: c_int = 2;
const EBUSY: c_int = 16;
const ENODEV: c_int = 19;
const EINVAL: c_int = 22;

fn check(result: c_int) -> Result<(), Error> {
    if result >= 0 {
        return Ok(());
    }
    // SAFETY: snd_strerror returns a static NUL-terminated string for any error code.
    let message = unsafe { CStr::from_ptr(sys::snd_strerror(result)) }
        .to_string_lossy()
        .into_owned();
    let kind = match -result {
        EBUSY => ErrorKind::DeviceBusy,
        ENOENT | ENODEV => ErrorKind::DeviceNotAvailable,
        EINVAL => ErrorKind::UnsupportedConfig,
        _ => ErrorKind::BackendError,
    };
    Err(Error::with_message(kind, message))
}

struct Pcm(NonNull<sys::SndPcm>);

// SAFETY: the PCM handle is only used by one thread at a time.
unsafe impl Send for Pcm {}

impl Pcm {
    fn open(mode: c_int) -> Result<Self, Error> {
        let mut raw = ptr::null_mut();
        // SAFETY: the name is NUL-terminated and `raw` is a valid output pointer.
        check(unsafe {
            sys::snd_pcm_open(
                &mut raw,
                DEVICE_NAME.as_ptr(),
                sys::SND_PCM_STREAM_CAPTURE,
                mode,
            )
        })?;
        NonNull::new(raw)
            .map(Self)
            .ok_or_else(|| Error::new(ErrorKind::DeviceNotAvailable))
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        // SAFETY: the handle was opened by snd_pcm_open and is closed once.
        unsafe { sys::snd_pcm_close(self.0.as_ptr()) };
    }
}

pub(crate) struct Device;

pub(crate) fn default_input_device() -> Option<Device> {
    Pcm::open(sys::SND_PCM_NONBLOCK).ok().map(|_| Device)
}

pub(crate) type Stream = capture::Stream;

impl Device {
    pub(crate) const fn default_sample_rate(&self) -> Result<SampleRate, Error> {
        Ok(DEFAULT_SAMPLE_RATE)
    }

    pub(crate) fn build_input_stream(
        &self,
        config: &StreamConfig,
        data: DataCallback,
        error: ErrorCallback,
    ) -> Result<Stream, Error> {
        let pcm = Pcm::open(0)?;
        let latency = config.buffer_frames() as u64 * 1_000_000 / config.sample_rate as u64;
        // SAFETY: the handle is open and all parameters are plain values.
        check(unsafe {
            sys::snd_pcm_set_params(
                pcm.0.as_ptr(),
                sys::SND_PCM_FORMAT_FLOAT_LE,
                sys::SND_PCM_ACCESS_RW_INTERLEAVED,
                config.channels as c_uint,
                config.sample_rate,
                1,
                latency as c_uint,
            )
        })?;
        let alsa = Alsa {
            pcm,
            channels: config.channels as usize,
            buffer: vec![0.0; config.buffer_frames() * config.channels as usize],
        };
        Stream::spawn(alsa, data, error)
    }
}

struct Alsa {
    pcm: Pcm,
    channels: usize,
    buffer: Vec<f32>,
}

impl Capture for Alsa {
    fn start(&mut self) -> Result<(), Error> {
        // SAFETY: the handle is open.
        check(unsafe { sys::snd_pcm_prepare(self.pcm.0.as_ptr()) })
    }

    fn stop(&mut self) -> Result<(), Error> {
        // SAFETY: the handle is open.
        check(unsafe { sys::snd_pcm_drop(self.pcm.0.as_ptr()) })
    }

    fn read(&mut self, samples: &mut Vec<f32>) -> Result<(), Error> {
        let frames = self.buffer.len() / self.channels;
        // SAFETY: the buffer holds `frames` interleaved frames of the configured format.
        let read = unsafe {
            sys::snd_pcm_readi(
                self.pcm.0.as_ptr(),
                self.buffer.as_mut_ptr().cast::<c_void>(),
                frames as c_ulong,
            )
        };
        if read < 0 {
            // Recover from overruns and suspends, other errors end the stream
            // SAFETY: the handle is open and `read` is the error of the last call.
            return check(unsafe { sys::snd_pcm_recover(self.pcm.0.as_ptr(), read as c_int, 1) });
        }
        samples.extend_from_slice(&self.buffer[..read as usize * self.channels]);
        Ok(())
    }
}
