/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;

use super::context::PlatformCanvasContext;
use super::headers::*;
use crate::copy_bgra_premultiplied_to_rgba;

pub(crate) struct PlatformOffscreenCanvas {
    surface: *mut c_void,
    context: *mut c_void,
    storage: Vec<u32>,
    output: Vec<u8>,
    dirty: bool,
}

impl PlatformOffscreenCanvas {
    pub(crate) fn new(width: u32, height: u32, length: usize) -> Self {
        let mut storage = vec![0; length / 4];
        unsafe {
            let surface = cairo_image_surface_create_for_data(
                storage.as_mut_ptr().cast(),
                0,
                width as i32,
                height as i32,
                (width * 4) as i32,
            );
            assert!(!surface.is_null(), "Could not create Cairo bitmap surface");
            let context = cairo_create(surface);
            assert_eq!(
                cairo_status(context),
                0,
                "Could not create Cairo bitmap canvas"
            );
            Self {
                surface,
                context,
                storage,
                output: vec![0; length],
                dirty: false,
            }
        }
    }

    pub(crate) fn begin_draw(&mut self) -> PlatformCanvasContext {
        self.dirty = true;
        unsafe { PlatformCanvasContext::new(self.context) }
    }

    pub(crate) const fn end_draw(&mut self) {}

    pub(crate) fn pixels(&mut self) -> &[u8] {
        if !self.dirty {
            return &self.output;
        }
        unsafe { cairo_surface_flush(self.surface) };
        let storage = unsafe {
            std::slice::from_raw_parts(self.storage.as_ptr().cast::<u8>(), self.output.len())
        };
        copy_bgra_premultiplied_to_rgba(storage, &mut self.output);
        self.dirty = false;
        &self.output
    }
}

impl Drop for PlatformOffscreenCanvas {
    fn drop(&mut self) {
        unsafe {
            cairo_destroy(self.context);
            cairo_surface_destroy(self.surface);
        }
    }
}
