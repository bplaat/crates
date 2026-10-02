/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! waveIn backend, the wave mapper converts the format, rate and channels.

#![allow(unsafe_code)]

use std::mem::size_of;
use std::ptr;

use crate::capture::{self, Capture, DEFAULT_SAMPLE_RATE};
use crate::{DataCallback, Error, ErrorCallback, ErrorKind, SampleRate, StreamConfig};

mod sys {
    use std::ffi::c_void;

    pub(super) type HWaveIn = *mut c_void;
    pub(super) type Handle = *mut c_void;

    pub(super) const WAVE_MAPPER: u32 = u32::MAX;
    pub(super) const WAVE_FORMAT_PCM: u16 = 1;
    pub(super) const CALLBACK_EVENT: u32 = 0x0005_0000;
    pub(super) const WHDR_DONE: u32 = 0x0000_0001;
    pub(super) const MMSYSERR_NOERROR: u32 = 0;
    pub(super) const MMSYSERR_BADDEVICEID: u32 = 2;
    pub(super) const MMSYSERR_ALLOCATED: u32 = 4;
    pub(super) const MMSYSERR_NODRIVER: u32 = 6;
    pub(super) const WAVERR_BADFORMAT: u32 = 32;
    pub(super) const MAXERRORLENGTH: usize = 256;

    #[repr(C, packed(1))]
    pub(super) struct WaveFormatEx {
        pub(super) format_tag: u16,
        pub(super) channels: u16,
        pub(super) samples_per_sec: u32,
        pub(super) avg_bytes_per_sec: u32,
        pub(super) block_align: u16,
        pub(super) bits_per_sample: u16,
        pub(super) size: u16,
    }

    #[repr(C)]
    pub(super) struct WaveHdr {
        pub(super) data: *mut u8,
        pub(super) buffer_length: u32,
        pub(super) bytes_recorded: u32,
        pub(super) user: usize,
        pub(super) flags: u32,
        pub(super) loops: u32,
        pub(super) next: *mut WaveHdr,
        pub(super) reserved: usize,
    }

    #[link(name = "winmm")]
    unsafe extern "system" {
        pub(super) fn waveInGetNumDevs() -> u32;
        pub(super) fn waveInOpen(
            handle: *mut HWaveIn,
            device_id: u32,
            format: *const WaveFormatEx,
            callback: usize,
            instance: usize,
            flags: u32,
        ) -> u32;
        pub(super) fn waveInClose(handle: HWaveIn) -> u32;
        pub(super) fn waveInPrepareHeader(handle: HWaveIn, header: *mut WaveHdr, size: u32) -> u32;
        pub(super) fn waveInUnprepareHeader(
            handle: HWaveIn,
            header: *mut WaveHdr,
            size: u32,
        ) -> u32;
        pub(super) fn waveInAddBuffer(handle: HWaveIn, header: *mut WaveHdr, size: u32) -> u32;
        pub(super) fn waveInStart(handle: HWaveIn) -> u32;
        pub(super) fn waveInStop(handle: HWaveIn) -> u32;
        pub(super) fn waveInReset(handle: HWaveIn) -> u32;
        pub(super) fn waveInGetErrorTextW(error: u32, text: *mut u16, length: u32) -> u32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub(super) fn CreateEventW(
            attributes: *const c_void,
            manual_reset: i32,
            initial_state: i32,
            name: *const u16,
        ) -> Handle;
        pub(super) fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
        pub(super) fn CloseHandle(handle: Handle) -> i32;
    }
}

/// Number of buffers queued at the wave mapper
const BUFFER_COUNT: usize = 4;
const WAIT_MILLISECONDS: u32 = 100;
const HEADER_SIZE: u32 = size_of::<sys::WaveHdr>() as u32;

fn check(result: u32) -> Result<(), Error> {
    if result == sys::MMSYSERR_NOERROR {
        return Ok(());
    }
    let mut text = [0u16; sys::MAXERRORLENGTH];
    // SAFETY: the buffer is valid for its length in UTF-16 units.
    unsafe { sys::waveInGetErrorTextW(result, text.as_mut_ptr(), text.len() as u32) };
    let length = text.iter().position(|&c| c == 0).unwrap_or(text.len());
    let kind = match result {
        sys::MMSYSERR_ALLOCATED => ErrorKind::DeviceBusy,
        sys::MMSYSERR_BADDEVICEID | sys::MMSYSERR_NODRIVER => ErrorKind::DeviceNotAvailable,
        sys::WAVERR_BADFORMAT => ErrorKind::UnsupportedConfig,
        _ => ErrorKind::BackendError,
    };
    Err(Error::with_message(
        kind,
        String::from_utf16_lossy(&text[..length]),
    ))
}

pub(crate) struct Device;

pub(crate) fn default_input_device() -> Option<Device> {
    // SAFETY: waveInGetNumDevs has no preconditions.
    (unsafe { sys::waveInGetNumDevs() } > 0).then_some(Device)
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
        Stream::spawn(WaveIn::open(config)?, data, error)
    }
}

struct WaveIn {
    handle: sys::HWaveIn,
    event: sys::Handle,
    /// Headers point into the buffers, both stay at a fixed address until dropped
    headers: Box<[sys::WaveHdr]>,
    _buffers: Box<[Box<[i16]>]>,
    /// The header that completes next
    next: usize,
}

// SAFETY: the wave handles are only used by one thread at a time.
unsafe impl Send for WaveIn {}

impl WaveIn {
    fn open(config: &StreamConfig) -> Result<Self, Error> {
        let block_align = config.channels * size_of::<i16>() as u16;
        let format = sys::WaveFormatEx {
            format_tag: sys::WAVE_FORMAT_PCM,
            channels: config.channels,
            samples_per_sec: config.sample_rate,
            avg_bytes_per_sec: config.sample_rate * block_align as u32,
            block_align,
            bits_per_sample: 16,
            size: 0,
        };

        // SAFETY: an unnamed auto-reset event without security attributes.
        let event = unsafe { sys::CreateEventW(ptr::null(), 0, 0, ptr::null()) };
        if event.is_null() {
            return Err(Error::new(ErrorKind::BackendError));
        }
        let mut handle = ptr::null_mut();
        // SAFETY: the format and output pointer are valid, the event outlives the handle.
        if let Err(err) = check(unsafe {
            sys::waveInOpen(
                &mut handle,
                sys::WAVE_MAPPER,
                &format,
                event as usize,
                0,
                sys::CALLBACK_EVENT,
            )
        }) {
            // SAFETY: the event was created above and is not used anymore.
            unsafe { sys::CloseHandle(event) };
            return Err(err);
        }

        let samples = config.buffer_frames() * config.channels as usize;
        let mut buffers: Box<[Box<[i16]>]> = (0..BUFFER_COUNT)
            .map(|_| vec![0; samples].into_boxed_slice())
            .collect();
        let headers = buffers
            .iter_mut()
            .map(|buffer| sys::WaveHdr {
                data: buffer.as_mut_ptr().cast(),
                buffer_length: (buffer.len() * size_of::<i16>()) as u32,
                bytes_recorded: 0,
                user: 0,
                flags: 0,
                loops: 0,
                next: ptr::null_mut(),
                reserved: 0,
            })
            .collect();
        let mut wave_in = Self {
            handle,
            event,
            headers,
            _buffers: buffers,
            next: 0,
        };
        for header in wave_in.headers.iter_mut() {
            // SAFETY: the handle is open and the header points to a live buffer.
            check(unsafe { sys::waveInPrepareHeader(handle, header, HEADER_SIZE) })?;
            // SAFETY: the header was prepared for this handle.
            check(unsafe { sys::waveInAddBuffer(handle, header, HEADER_SIZE) })?;
        }
        Ok(wave_in)
    }
}

impl Drop for WaveIn {
    fn drop(&mut self) {
        // SAFETY: reset returns all buffers, so they can be unprepared before closing.
        unsafe {
            sys::waveInReset(self.handle);
            for header in self.headers.iter_mut() {
                sys::waveInUnprepareHeader(self.handle, header, HEADER_SIZE);
            }
            sys::waveInClose(self.handle);
            sys::CloseHandle(self.event);
        }
    }
}

impl Capture for WaveIn {
    fn start(&mut self) -> Result<(), Error> {
        // SAFETY: the handle is open.
        check(unsafe { sys::waveInStart(self.handle) })
    }

    fn stop(&mut self) -> Result<(), Error> {
        // SAFETY: the handle is open.
        check(unsafe { sys::waveInStop(self.handle) })
    }

    fn read(&mut self, samples: &mut Vec<f32>) -> Result<(), Error> {
        let header: *mut sys::WaveHdr = &mut self.headers[self.next];
        // SAFETY: the driver sets the done flag from another thread, so it is read volatile.
        if unsafe { ptr::read_volatile(&raw const (*header).flags) } & sys::WHDR_DONE == 0 {
            // SAFETY: the event is open, a timeout lets the capture thread see state changes.
            unsafe { sys::WaitForSingleObject(self.event, WAIT_MILLISECONDS) };
            return Ok(());
        }

        // SAFETY: a done header is owned by us again until it is added back.
        let header = unsafe { &mut *header };
        let length = header.bytes_recorded as usize / size_of::<i16>();
        // SAFETY: the driver recorded `bytes_recorded` bytes of 16-bit samples into the buffer.
        let recorded = unsafe { std::slice::from_raw_parts(header.data.cast::<i16>(), length) };
        samples.extend(recorded.iter().map(|&sample| sample as f32 / 32768.0));
        self.next = (self.next + 1) % self.headers.len();
        // SAFETY: the header is prepared for this handle and its buffer is live.
        check(unsafe { sys::waveInAddBuffer(self.handle, header, HEADER_SIZE) })
    }
}
