/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! AudioQueue backend, the queue converts the format, rate and channels.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use crate::{DataCallback, Error, ErrorCallback, ErrorKind, SampleRate, StreamConfig};

mod sys {
    use std::ffi::c_void;

    pub(super) type OsStatus = i32;
    pub(super) type AudioObjectId = u32;
    pub(super) type AudioQueueRef = *mut c_void;
    pub(super) type AudioQueueInputCallback = unsafe extern "C" fn(
        user_data: *mut c_void,
        queue: AudioQueueRef,
        buffer: *mut AudioQueueBuffer,
        start_time: *const c_void,
        packet_count: u32,
        packet_descriptions: *const c_void,
    );

    const fn four_cc(code: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*code)
    }

    pub(super) const K_AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectId = 1;
    pub(super) const K_AUDIO_OBJECT_UNKNOWN: AudioObjectId = 0;
    pub(super) const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE: u32 = four_cc(b"dIn ");
    pub(super) const K_AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE: u32 = four_cc(b"nsrt");
    pub(super) const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = four_cc(b"glob");
    pub(super) const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0;
    pub(super) const K_AUDIO_FORMAT_LINEAR_PCM: u32 = four_cc(b"lpcm");
    pub(super) const K_AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1 << 0;
    pub(super) const K_AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 1 << 3;

    #[repr(C)]
    pub(super) struct AudioObjectPropertyAddress {
        pub(super) selector: u32,
        pub(super) scope: u32,
        pub(super) element: u32,
    }

    #[repr(C)]
    pub(super) struct AudioStreamBasicDescription {
        pub(super) sample_rate: f64,
        pub(super) format_id: u32,
        pub(super) format_flags: u32,
        pub(super) bytes_per_packet: u32,
        pub(super) frames_per_packet: u32,
        pub(super) bytes_per_frame: u32,
        pub(super) channels_per_frame: u32,
        pub(super) bits_per_channel: u32,
        pub(super) reserved: u32,
    }

    #[repr(C)]
    pub(super) struct AudioQueueBuffer {
        pub(super) audio_data_bytes_capacity: u32,
        pub(super) audio_data: *mut c_void,
        pub(super) audio_data_byte_size: u32,
        pub(super) user_data: *mut c_void,
        pub(super) packet_description_capacity: u32,
        pub(super) packet_descriptions: *mut c_void,
        pub(super) packet_description_count: u32,
    }

    #[link(name = "CoreAudio", kind = "framework")]
    unsafe extern "C" {
        pub(super) fn AudioObjectGetPropertyData(
            object: AudioObjectId,
            address: *const AudioObjectPropertyAddress,
            qualifier_size: u32,
            qualifier: *const c_void,
            data_size: *mut u32,
            data: *mut c_void,
        ) -> OsStatus;
    }

    #[link(name = "AudioToolbox", kind = "framework")]
    unsafe extern "C" {
        pub(super) fn AudioQueueNewInput(
            format: *const AudioStreamBasicDescription,
            callback: AudioQueueInputCallback,
            user_data: *mut c_void,
            run_loop: *const c_void,
            run_loop_mode: *const c_void,
            flags: u32,
            queue: *mut AudioQueueRef,
        ) -> OsStatus;
        pub(super) fn AudioQueueAllocateBuffer(
            queue: AudioQueueRef,
            byte_size: u32,
            buffer: *mut *mut AudioQueueBuffer,
        ) -> OsStatus;
        pub(super) fn AudioQueueEnqueueBuffer(
            queue: AudioQueueRef,
            buffer: *mut AudioQueueBuffer,
            packet_count: u32,
            packet_descriptions: *const c_void,
        ) -> OsStatus;
        pub(super) fn AudioQueueStart(queue: AudioQueueRef, start_time: *const c_void) -> OsStatus;
        pub(super) fn AudioQueuePause(queue: AudioQueueRef) -> OsStatus;
        pub(super) fn AudioQueueDispose(queue: AudioQueueRef, immediate: u8) -> OsStatus;
    }
}

/// Number of buffers queued at the audio queue
const BUFFER_COUNT: usize = 3;

fn check(status: sys::OsStatus) -> Result<(), Error> {
    if status == 0 {
        Ok(())
    } else {
        Err(Error::with_message(
            ErrorKind::BackendError,
            format!("CoreAudio error {status}"),
        ))
    }
}

/// Reads a fixed size property of a CoreAudio object
fn property<T: Default>(object: sys::AudioObjectId, selector: u32) -> Result<T, Error> {
    let address = sys::AudioObjectPropertyAddress {
        selector,
        scope: sys::K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
        element: sys::K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
    };
    let mut value = T::default();
    let mut size = size_of::<T>() as u32;
    // SAFETY: the address is valid and the value has room for `size` bytes.
    check(unsafe {
        sys::AudioObjectGetPropertyData(
            object,
            &address,
            0,
            ptr::null(),
            &mut size,
            (&raw mut value).cast(),
        )
    })?;
    Ok(value)
}

pub(crate) struct Device(sys::AudioObjectId);

pub(crate) fn default_input_device() -> Option<Device> {
    property(
        sys::K_AUDIO_OBJECT_SYSTEM_OBJECT,
        sys::K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE,
    )
    .ok()
    .filter(|&id| id != sys::K_AUDIO_OBJECT_UNKNOWN)
    .map(Device)
}

impl Device {
    pub(crate) fn default_sample_rate(&self) -> Result<SampleRate, Error> {
        property::<f64>(self.0, sys::K_AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE)
            .map(|rate| rate as SampleRate)
    }

    /// The audio queue records from the default input device, errors are reported when building
    pub(crate) fn build_input_stream(
        &self,
        config: &StreamConfig,
        data: DataCallback,
        _error: ErrorCallback,
    ) -> Result<Stream, Error> {
        let channels = config.channels as u32;
        let format = sys::AudioStreamBasicDescription {
            sample_rate: config.sample_rate as f64,
            format_id: sys::K_AUDIO_FORMAT_LINEAR_PCM,
            format_flags: sys::K_AUDIO_FORMAT_FLAG_IS_FLOAT | sys::K_AUDIO_FORMAT_FLAG_IS_PACKED,
            bytes_per_packet: channels * size_of::<f32>() as u32,
            frames_per_packet: 1,
            bytes_per_frame: channels * size_of::<f32>() as u32,
            channels_per_frame: channels,
            bits_per_channel: 32,
            reserved: 0,
        };

        let data = Box::into_raw(Box::new(data));
        let mut queue = ptr::null_mut();
        // SAFETY: without a run loop the callback runs on the queue's own thread, `data` stays
        // alive until the queue is disposed.
        if let Err(err) = check(unsafe {
            sys::AudioQueueNewInput(
                &format,
                input_callback,
                data.cast(),
                ptr::null(),
                ptr::null(),
                0,
                &mut queue,
            )
        }) {
            // SAFETY: the queue wasn't created, so nothing else owns `data`.
            drop(unsafe { Box::from_raw(data) });
            return Err(err);
        }
        let stream = Stream { queue, data };

        let byte_size =
            (config.buffer_frames() * config.channels as usize * size_of::<f32>()) as u32;
        for _ in 0..BUFFER_COUNT {
            let mut buffer = ptr::null_mut();
            // SAFETY: the queue is live and owns the allocated buffer.
            check(unsafe { sys::AudioQueueAllocateBuffer(queue, byte_size, &mut buffer) })?;
            // SAFETY: the buffer belongs to this queue.
            check(unsafe { sys::AudioQueueEnqueueBuffer(queue, buffer, 0, ptr::null()) })?;
        }
        Ok(stream)
    }
}

unsafe extern "C" fn input_callback(
    user_data: *mut c_void,
    queue: sys::AudioQueueRef,
    buffer: *mut sys::AudioQueueBuffer,
    _start_time: *const c_void,
    _packet_count: u32,
    _packet_descriptions: *const c_void,
) {
    // SAFETY: the queue passes the callback and a filled buffer of packed f32 frames, callbacks
    // run one at a time on the queue's thread.
    unsafe {
        let data = &mut *user_data.cast::<DataCallback>();
        let buffer_ref = &*buffer;
        let samples = std::slice::from_raw_parts(
            buffer_ref.audio_data.cast::<f32>(),
            buffer_ref.audio_data_byte_size as usize / size_of::<f32>(),
        );
        if !samples.is_empty() {
            data(samples);
        }
        // Fails while the queue is disposed, then the buffer is freed with it
        sys::AudioQueueEnqueueBuffer(queue, buffer, 0, ptr::null());
    }
}

pub(crate) struct Stream {
    queue: sys::AudioQueueRef,
    data: *mut DataCallback,
}

// SAFETY: audio queue functions are thread-safe and the callback is only used by the queue.
unsafe impl Send for Stream {}
// SAFETY: the same audio queue thread-safety permits shared references.
unsafe impl Sync for Stream {}

impl Stream {
    pub(crate) fn play(&self) -> Result<(), Error> {
        // SAFETY: the queue is live.
        check(unsafe { sys::AudioQueueStart(self.queue, ptr::null()) })
    }

    pub(crate) fn pause(&self) -> Result<(), Error> {
        // SAFETY: the queue is live.
        check(unsafe { sys::AudioQueuePause(self.queue) })
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: an immediate dispose waits for running callbacks, so `data` can be freed after.
        unsafe {
            sys::AudioQueueDispose(self.queue, 1);
            drop(Box::from_raw(self.data));
        }
    }
}
